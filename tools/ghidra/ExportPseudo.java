// Ghidra headless post-script: export 6502 C pseudocode projection (S1).
// Projection only; ROM bytes are truth (D3). Deterministic output: no timestamps.
//
// Usage (via tools/ghidra/run_all_banks.sh):
//   analyzeHeadless <projDir> s1 -import bankNN.bin -loader BinaryLoader \
//     -processor 6502:LE:16:default -loader-baseAddr 0x8000 \
//     -scriptPath tools/ghidra -postScript ExportPseudo.java <out.c>
//
// Mechanics: byte-scan JSR($20)/JMP($4C) absolute targets inside the loaded bank,
// disassemble + create functions there, then decompile every function to C.

import ghidra.app.decompiler.DecompInterface;
import ghidra.app.decompiler.DecompileResults;
import ghidra.app.script.GhidraScript;
import ghidra.program.model.address.Address;
import ghidra.program.model.listing.Function;
import ghidra.program.model.listing.FunctionIterator;
import ghidra.program.model.listing.FunctionManager;
import ghidra.program.model.mem.Memory;
import ghidra.program.model.mem.MemoryBlock;

import java.io.FileWriter;
import java.util.TreeSet;

public class ExportPseudo extends GhidraScript {

    @Override
    public void run() throws Exception {
        String outPath = getScriptArgs()[0];
        Memory mem = currentProgram.getMemory();
        Address min = mem.getMinAddress();
        Address max = mem.getMaxAddress();

        // 1) mechanical entry discovery: JSR/JMP absolute targets inside this bank
        TreeSet<Long> entries = new TreeSet<Long>();
        for (MemoryBlock block : mem.getBlocks()) {
            if (!block.isInitialized()) {
                continue;
            }
            long size = block.getSize();
            byte[] data = new byte[(int) size];
            block.getBytes(block.getStart(), data);
            long base = block.getStart().getOffset();
            for (int i = 0; i + 2 < data.length; i++) {
                int op = data[i] & 0xFF;
                if (op == 0x20 || op == 0x4C) {
                    long tgt = (data[i + 1] & 0xFF) | ((data[i + 2] & 0xFF) << 8);
                    if (tgt >= min.getOffset() && tgt <= max.getOffset()) {
                        entries.add(tgt);
                    }
                }
            }
        }
        entries.add(min.getOffset());

        // 2) disassemble + create functions at mechanical entries
        for (long ea : entries) {
            Address addr = getAddressFactory().getDefaultAddressSpace().getAddress(ea);
            if (addr.compareTo(min) < 0 || addr.compareTo(max) > 0) {
                continue;
            }
            disassemble(addr);
            createFunction(addr, null);
        }
        analyzeAll(currentProgram);

        // 3) decompile all functions to C
        DecompInterface decomp = new DecompInterface();
        decomp.openProgram(currentProgram);
        StringBuilder sb = new StringBuilder();
        sb.append("// Ghidra 6502 C pseudocode projection (analysis artifact, regenerable)\n");
        sb.append("// program: ").append(currentProgram.getName())
          .append("  base: ").append(min).append("\n");
        FunctionManager fm = currentProgram.getFunctionManager();
        int n = 0;
        for (FunctionIterator it = fm.getFunctions(true); it.hasNext(); ) {
            Function f = it.next();
            n++;
            sb.append("// ===== ").append(f.getName()).append(" @ ")
              .append(f.getEntryPoint()).append(" =====\n");
            DecompileResults res = decomp.decompileFunction(f, 60, monitor);
            if (res != null && res.decompileCompleted() && res.getDecompiledFunction() != null) {
                sb.append(res.getDecompiledFunction().getC());
            } else {
                sb.append("// decompile failed: ")
                  .append(res == null ? "null" : res.getErrorMessage()).append("\n");
            }
            sb.append("\n");
        }
        decomp.dispose();
        FileWriter fw = new FileWriter(outPath);
        fw.write(sb.toString());
        fw.close();
        println("S1 ghidra export: " + n + " functions -> " + outPath);
    }
}

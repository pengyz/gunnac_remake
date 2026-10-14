-- C2 段级接力实证 v3：createSavestate 须在 exec 回调内（v2 错误信息实证）→ 用 NMI 向量 exec 钩
local out = "/tmp/c2_relay_probe2.out"
local f = io.open(out, "w")
local function w(s) f:write(s .. "\n") f:flush() end
w("start")
local frame, nmi = 0, 0
local saved, savedFrame, savedPx, savedPy, phase = nil, nil, nil, nil, "run"
pcall(emu.addEventCallback, function() frame = frame + 1 pcall(emu.setInput, {}, 0) end, "inputPolled")
local okcb, errcb = pcall(emu.addMemoryCallback, function(addr, value)
  nmi = nmi + 1
  if nmi == 100 and phase == "run" then
    local oks, sv = pcall(emu.createSavestate)
    w("createSavestate ok=" .. tostring(oks) .. " type=" .. type(sv))
    if oks then
      saved = sv
      savedFrame = emu.getState()["frameCount"]
      savedPx = emu.read(0x04E0, 8)
      savedPy = emu.read(0x04FC, 8)
      w(string.format("SAVED nmi=%d frameCount=%s px=%d py=%d", nmi, tostring(savedFrame), savedPx, savedPy))
      phase = "saved"
    else
      w("createSavestate err=" .. tostring(sv))
      phase = "fail"
    end
  elseif nmi == 300 and phase == "saved" then
    w(string.format("BEFORE-LOAD nmi=%d frameCount=%s px=%d py=%d", nmi,
      tostring(emu.getState()["frameCount"]), emu.read(0x04E0, 8), emu.read(0x04FC, 8)))
    local okl, errl = pcall(emu.loadSavestate, saved)
    w("loadSavestate ok=" .. tostring(okl) .. " err=" .. tostring(errl))
    phase = okl and "loaded" or "fail"
  elseif nmi == 305 and phase == "loaded" then
    w(string.format("AFTER-LOAD nmi=%d frameCount=%s px=%d py=%d savedPx=%d savedPy=%d savedFrame=%s",
      nmi, tostring(emu.getState()["frameCount"]), emu.read(0x04E0, 8), emu.read(0x04FC, 8),
      savedPx, savedPy, tostring(savedFrame)))
    phase = "done"
    f:close()
    pcall(emu.stop)
  end
end, "exec", 0xEF85, 0xEF85)
w("exec cb ok=" .. tostring(okcb) .. " err=" .. tostring(errcb))

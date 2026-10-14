-- C2 save-state / C1 RNG 观测面能力探测（Mesen2 2.1.1）——只观测与能力枚举
-- 纪律：本脚本属**探针**（非通关场景）；不写 ROM/不注入输入；setState/saveState 能力
-- 只做 round-trip 自检（自身状态写回自身），产出标注 trigger=probe。
local out = "/tmp/c2_api_probe.out"
local f = io.open(out, "w")
local function w(s) f:write(s .. "\n") end
local ok, err = pcall(function()
  local keys = {}
  local ok2, err2 = pcall(function()
    for k, v in pairs(emu) do keys[#keys + 1] = k .. "=" .. type(v) end
  end)
  if not ok2 then w("pairs(emu) FAILED: " .. tostring(err2)) end
  table.sort(keys)
  w("emu keys(" .. #keys .. "): " .. table.concat(keys, ", "))
end)
if not ok then w("enumeration FAILED: " .. tostring(err)) end
for _, n in ipairs({"getState","setState","saveStateFile","loadStateFile","saveState",
                    "loadState","read","write","takeScreenshot","stop","addEventCallback",
                    "addMemoryCallback","getMouseState","log","message"}) do
  w(string.format("emu.%s = %s", n, tostring(emu[n])))
end
-- getState 结构抽样
local ok3, st = pcall(emu.getState)
w("getState ok=" .. tostring(ok3) .. " type=" .. type(st))
if ok3 and type(st) == "table" then
  local ks = {}
  for k, v in pairs(st) do ks[#ks + 1] = tostring(k) .. ":" .. type(v) end
  table.sort(ks)
  w("state keys(" .. #ks .. "): " .. table.concat(ks, ", "))
end
-- setState round-trip 自检（自身状态写回自身 = 不引入外部状态；能力验证）
if ok3 and emu.setState then
  local ok4, err4 = pcall(emu.setState, st)
  w("setState(getState()) roundtrip ok=" .. tostring(ok4) .. " err=" .. tostring(err4))
end
-- saveStateFile/loadStateFile 能力验证（若存在）
if emu.saveStateFile then
  local ok5, err5 = pcall(emu.saveStateFile, "/tmp/c2_api_probe.state")
  w("saveStateFile ok=" .. tostring(ok5) .. " err=" .. tostring(err5))
end
f:close()

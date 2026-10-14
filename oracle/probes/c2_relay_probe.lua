-- C2 段级接力能力实证 v2（全 pcall + 逐段日志）
local out = "/tmp/c2_relay_probe.out"
local f = io.open(out, "w")
local function w(s) f:write(s .. "\n") f:flush() end
w("start")
local frame = 0
local saved, savedFrame, savedPx, savedPy = nil, nil, nil, nil
local phase = "run"
local ok1, err1 = pcall(emu.addEventCallback, function()
  frame = frame + 1
  pcall(emu.setInput, {}, 0)
end, "inputPolled")
w("cb inputPolled ok=" .. tostring(ok1) .. " err=" .. tostring(err1))
local ok2, err2 = pcall(emu.addEventCallback, function()
  local okr, px = pcall(emu.read, 0x04E0, 8)
  local _, py = pcall(emu.read, 0x04FC, 8)
  if frame % 50 == 0 then w("tick frame=" .. frame .. " px=" .. tostring(px)) end
  if frame == 100 and phase == "run" then
    local oks, sv = pcall(emu.createSavestate)
    w("createSavestate ok=" .. tostring(oks) .. " type=" .. type(sv) .. " err=" .. tostring(sv))
    if oks then
      saved = sv
      savedFrame = emu.getState()["frameCount"]
      savedPx, savedPy = px, py
      phase = "saved"
    end
  elseif frame == 200 and phase == "saved" then
    w(string.format("BEFORE-LOAD frame=%d px=%s py=%s", frame, tostring(px), tostring(py)))
    local okl, errl = pcall(emu.loadSavestate, saved)
    w("loadSavestate ok=" .. tostring(okl) .. " err=" .. tostring(errl))
    phase = "loaded"
  elseif phase == "loaded" then
    local fc = emu.getState()["frameCount"]
    w(string.format("AFTER-LOAD frame=%d frameCount=%s px=%s py=%s savedPx=%s savedPy=%s savedFrame=%s",
      frame, tostring(fc), tostring(px), tostring(py), tostring(savedPx), tostring(savedPy), tostring(savedFrame)))
    phase = "done"
    pcall(emu.stop)
  end
end, "endFrame")
w("cb endFrame ok=" .. tostring(ok2) .. " err=" .. tostring(err2))

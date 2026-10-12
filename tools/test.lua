local frame = 0
emu.speedmode("maximum")
while true do
  frame = frame + 1
  if frame == 600 then gui.savescreenshotas("/home/peng/workspace/gunnac_remake/shots/test_600.png") end
  if frame == 1200 then
    gui.savescreenshotas("/home/peng/workspace/gunnac_remake/shots/test_1200.png")
    break
  end
  emu.frameadvance()
end

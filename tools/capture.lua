-- Gun-Nac auto-boot + screenshot capture for FCEUX
-- Boots the ROM, presses START through menus, captures frames periodically.
local dir = "/home/peng/workspace/gunnac_remake/shots/"
local frame = 0

emu.speedmode("maximum")

local function shot(tag)
  gui.savescreenshotas(string.format("%s%s_f%05d.png", dir, tag, frame))
end

while true do
  frame = frame + 1

  -- default: no buttons
  joypad.set(1, {start=false, select=false, A=false, B=false,
                 up=false, down=false, left=false, right=false})

  -- spam START to get through title / mode / difficulty menus
  if (frame >= 240 and frame <= 246)
  or (frame >= 300 and frame <= 306)
  or (frame >= 360 and frame <= 366)
  or (frame >= 420 and frame <= 426)
  or (frame >= 600 and frame <= 606)
  or (frame >= 660 and frame <= 666)
  or (frame >= 900 and frame <= 906)
  or (frame >= 1200 and frame <= 1206) then
    joypad.set(1, {start=true})
  end

  -- periodic captures
  if frame % 180 == 0 then
    shot("auto")
  end

  if frame >= 5400 then
    shot("auto")
    emu.frameadvance()
    -- stop the script; process is killed by timeout anyway
    break
  end

  emu.frameadvance()
end

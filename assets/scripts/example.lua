-- Script de exemplo — Engine Studio / Lua 5.4
-- Anexado a entidades na cena (campo "script" no JSON)

local spin = 0.0

function on_start()
    engine.log("Script iniciado — areia e sol ativos!")
end

function update(dt)
    spin = spin + dt * 0.5
    if spin > 6.28 then
        spin = 0.0
        engine.log(string.format("Tempo de cena: %.1fs", engine.time()))
    end
end

function on_stop()
    engine.log("Play mode encerrado.")
end

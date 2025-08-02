task_run("g++ --version", function(line)
    if string.find(line, "ERROR") then
        print_error("Compiler error: " .. line)
        return false
    else
        print(line)
        return true
    end
end)




task_realtime("ping -n 10 underko.ru", function(data)
    if data.line then
        print("Ping: " .. data.line)
    elseif data.final_output then
        print("Ping complete!")
    elseif data.error then
        print("Error: " .. data.error)
    end
end)
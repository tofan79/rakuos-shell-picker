HOME = os.getenv("HOME") or ""

function is_file_exists(name)
    local f = io.open(name, "r")
    if f ~= nil then
        io.close(f)
        return true
    end
    return false
end

-- The first command of `list` that is installed, or nil. One `sh` for the whole list,
-- so a config reload does not spawn a process per candidate.
function first_available(list)
    local script = 'for c in ' .. table.concat(list, " ")
        .. '; do command -v "$c" >/dev/null 2>&1 && { echo "$c"; break; }; done'
    local ok, pipe = pcall(io.popen, "sh -c '" .. script .. "' 2>/dev/null")
    if not ok or not pipe then
        return nil
    end
    local found = pipe:read("*l")
    pipe:close()
    if found and found ~= "" then
        return found
    end
    return nil
end

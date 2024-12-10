local VIDEO_EXTENSIONS = {"mp4", "mkv", "avi", "webm", "flv", "mov", "wmv"}

function is_video_file(file)
    local extention = file:match("^.+%.(.+)$")
    
    for _, ext in ipairs(VIDEO_EXTENSIONS) do
        if ext == extention then
            return true
        end
    end

    return false
end

function join_paths(path1, path2)
    if path2:match("^[A-Za-z]:[/\\]") then
        return path2
    end

    path1 = path1:gsub("[/\\]+$", "")
    path2 = path2:gsub("^[/\\]+", "")
    
    return path1 .. "/" .. path2
end

function on_file_loaded(event)
    -- Get the extention and folder of the file
    local working_directory = mp.get_property("working-directory"):gsub("\\", "/")
    local file = mp.get_property("filename")
    local path = join_paths(working_directory, mp.get_property("path"))

    local folder = path:match("^(.*[\\/])")
    local extention = file:match("^.+%.(.+)$")
    if extention == nil or folder == nil or file == nil or not is_video_file(file) then
        return
    end

    -- Load the sidecar file
    local sidecar_path = join_paths(folder, ".watched")
    local success, lines = pcall(io.lines, sidecar_path)

    -- Check for the current file in the sidecar file, returning if it is already there
    if success then
        for line in lines do
            if line == file then
                mp.osd_message("Already watched")
                return
            end
        end
    end
    
    mp.osd_message("Marking as watched")
    local sidecar = io.open(sidecar_path, "a+")
    sidecar:write(file .. "\n")
    sidecar:close()
end

mp.register_event("file-loaded", on_file_loaded)
red.buffer:set_type(red.buffer.naive)
red.buffer:link_file(red.file:open("../profiling/large_file/file.txt"))
red.buffer:set_cursor_line(500000)

for _=1,10000 do
    red.buffer:insert_at_cursor("\nI")
end
red.exit()

-- This file is part of BadRed.

-- BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
--
-- BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

package.preload["opts"] = function(_, _)
    local doc = require("doc")

    local P = doc.document_table(
        {},
        "opts",
        "Collection of editor options and values which customize editor behavior.",
        nil,
        {},
        function(v) return v end
    )

    doc.add_documented_field(
        P,
        "expand_tabs",
        false,
        "If true, will replace all tabs input with the corresponding number of spaces."
    )

    doc.add_documented_field(
        P,
        "tab_width",
        8,
        "The number of spaces a single tab is shown."
    )

    return P
end

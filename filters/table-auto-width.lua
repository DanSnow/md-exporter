-- Replace explicit ColWidth with ColWidthDefault so Typst uses auto column sizing
function Table(tbl)
  for i, spec in ipairs(tbl.colspecs) do
    tbl.colspecs[i] = { spec[1], pandoc.ColWidthDefault }
  end
  return tbl
end

function pairs(tab)
  local fuck = 0
  local fuck_tab = tab
  return function()
    local res = fuck_tab[fuck]
    fuck = fuck + 1
    return res, fuck
  end
end

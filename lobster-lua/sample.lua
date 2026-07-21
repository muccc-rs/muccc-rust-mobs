



-- this is a comment

--[===[ Multiline

Line 2
]===]



-- return nil, nil;

foo = 1
bar = XII
baz = IVIVIV

something = foo + bar

var = [[Hello]]
print([[Hello world]], MIMIMIMIMI)

hugo = (5 + (MIMIMIMIMI - IMIVIIII))

-- abc = (((((print)))))([[Hello]])

var_again = var

if var_again == [[Hello]] then
	hugo = MIMIMI
elseif hugo == MIMIMI then
	bangkok = true
end

mimimi = MIMIMI
is_this_mimimi = (5 + (MIMIMIMIMI - IMIVIIII))

i = 12
res = 1
while i ~= 0 do
	res = res * i
	i = i - 1
end

print([[are you alive?]])
foobar = [[foo]] .. [[bar]]

print([[All the fractions]])
print(·)
-- print(:)
print(∴)
print(∷)
print(⁙)
print(S)
print(S·)
print(S:)
print(S∴)
print()
print([[zzz]], S∷ / 2)

my_function = function ()
	print([[hello from my function]])
end
my_function()

function my_function_2 ()
  print([[test]])
end

my_local = 0

function my_function_2 ()
  local my_local = 2
  print(my_local)
  print([[test]])
end

my_function_2()

print(my_local)

print([[Addition:]])
print(1/3 + 1/2)

print([[Returning:]])

function my_function_3 ()
  return [[something]]
--  print ([[print after return]]) -- NO EARLY RETURN!!!
end

my_test_val = my_function_3 ()
print ([[my test value after return:]])
print (my_test_val)


c = 0

add = function (a, b)
  c = a + b
  return c
end

add(1,2)

add(·,2)
print(c)
add(2,·)
print(c)
add(·,·)

print(c)

print(1/2)

-- local string = [[string]] .. [=[      ]=]

-- k = DCIXX -- Very nice

-- local meaning_of_life = 21 * 2 - XI

x = [[y]]
t = { x = 1, [x] = 2 }
print(t)

l = { 3, 2, 1 }
print(l)

print(t[ [[x]] ])
print(l[1])
print(l[2])

l[2] = 42
print([[lllllllllllllllll: ]], l)
t2 = { a = { b = 42 } , test = function(self,arg)
  print([[test]],self.a.b)
end
}
print(t2.a.b)
print(t2:test(foo))
-- print(t2.a(t2, foo))


too = { a = {b = add} }
food = too[  [[a]]  ].b(1,2)
too[  [[a]]  ].b(1,2)
too.a.b = 2
print(food)
print(too)

string.split = function(str) 
  return string.gmatch(str, [==[([^ ])]==])
end

function foo ()
		 return too
end

three = { t = foo }

three.t().a.b = 4

print(too)

print2 = print
print2([[hello]])

os.execute([[uname -a]])
-- TODO
--f = io.open([[/home/rahix/.ssh/id_rsa]], [[r]])
--content = f:read(34000)
--print(content)

local v = string.split([[foo bar baz]])

-- BUSTED, i assignment in loop body seems to write to non-local?
--local i = 0
i = 0 
while i ~= 3 do
  print(i, v[i])
  i = i + 1
end

listener = io.bind([[127.0.0.1:1234]])
while false ~= true do
  print([[Accepting...]])
  stream = listener:accept()
  print([[Accepted!...]])
  child_pid = os.fork()
  if child_pid == 0 then
  	print([[I am the child]])
    message = stream:read(128)
    print([[Received]])
    parts = string.split(message)
    if parts[0] == [[GET]] then
      url = parts[1]
      resp = [[<b>To Whom It May Concern</b>,<br />
<br />
you have hereby succesfuly requested the following file:<br />
<br />
]] .. url .. [[<br />
<br /> 
Kind Greetings,<br />
Your 'Lua' Server]]
      len = string.len(resp)

      stream:write([[HTTP/1.1 200 OK
Server: lau
Connection: close
Content-Length: ]] .. len .. [[

]] .. resp .. [[
]])

    end
    stream:close()
    print([[Goodbye]])
    break
  end
end
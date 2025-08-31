
function Main(...)
	print("Hello from lua main!!!")

	for k,v in pairs(ctx) do
		print(k,v)
		if k == "data" then
			print("is data")
			for _,n in pairs(v) do
				print 'any'
				print(n)
				print(math.random())
			end
		end
	end

	local reviewCount = ctx.fn('message from lua')

	if reviewCount > 1000 then
		print(reviewCount.." is more 1000")
	else
		print('---')
	end

	print(ctx.fn('message from lua'))
end

Main()

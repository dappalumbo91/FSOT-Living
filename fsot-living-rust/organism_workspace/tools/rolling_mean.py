import sys
xs=[float(x) for x in sys.argv[1:]]
print(0 if not xs else sum(xs)/len(xs))

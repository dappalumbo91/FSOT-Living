import sys
x=float(sys.argv[1]) if len(sys.argv)>1 else 0
print(max(0.0,min(1.0,x)))

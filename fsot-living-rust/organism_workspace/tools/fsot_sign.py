import sys
x=float(sys.argv[1]) if len(sys.argv)>1 else 0
print('emergence' if x>0 else 'damping' if x<0 else 'neutral')

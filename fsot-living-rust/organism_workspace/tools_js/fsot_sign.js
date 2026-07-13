const x=Number(process.argv[2]||0);
console.log(x>0?'emergence':x<0?'damping':'neutral');

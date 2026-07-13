const a=process.argv.slice(2).map(Number);
const x=a[0]||0, lo=a[1]||0, hi=(a[2]===undefined?1:a[2]);
console.log(Math.max(lo,Math.min(hi,x)));

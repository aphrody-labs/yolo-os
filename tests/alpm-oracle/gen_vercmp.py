import subprocess, random, json, sys
random.seed(1)
vers = subprocess.run(["pacman","-Q"],capture_output=True,text=True).stdout.split("\n")
vs = sorted({l.split()[1] for l in vers if l.strip()})
edge = ["1.0","1.0.0","1.0a","1.0b","1.0rc1","1.0.1","1:1.0","0:1.0","2:0.1","1.0-1","1.0-2","1.0-1.1","1.0_1","1.0+1","1.a","1.1","1a","a","1","1..0","1.0.","","~1","1.0~rc1","r100.abc","20240101","2024.01.01","1.0alpha","1.0beta","1.0.beta","1.0.0.0","001","1","01","1.0-1.0","1.0-1.a"]
pool = vs + edge
pairs=set()
while len(pairs)<2400:
    a=random.choice(pool); 
    b=random.choice(pool) if random.random()<.5 else random.choice(vs)
    pairs.add((a,b))
out=[]
for a,b in sorted(pairs):
    r=subprocess.run(["vercmp",a,b],capture_output=True,text=True)
    out.append({"a":a,"b":b,"result":int(r.stdout.strip())})
json.dump({"generator":"vercmp","pairs":out},open(sys.argv[1],"w"),separators=(",",":"))
print(len(out))

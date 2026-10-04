import json,subprocess,time,sys,random,statistics
model=sys.argv[1]; ns=[int(x) for x in sys.argv[2].split(',')]; reps=int(sys.argv[3]) if len(sys.argv)>3 else 3
base=json.load(open('req12.json')); items=list(base['questions'].items())
fn='tmp_%s.json'%model.replace(':','_')
def call(n):
    b=dict(base); b['model']=model; b['questions']=dict(items[:n])
    b['state']=base['state']+f" (ref {random.randint(10**6,10**7)})"   # bust any prompt cache
    json.dump(b,open(fn,'w'))
    t=time.time()
    out=subprocess.run(['curl','-s','-m','900','http://localhost:11434/v1/systemone','-d','@'+fn],capture_output=True,text=True).stdout
    d=json.loads(out); return time.time()-t,d
call(1)
for n in ns:
    ts=[];tok=0
    for _ in range(reps):
        dt,d=call(n); ts.append(dt); tok=d['usage']['input_tokens']
    m=statistics.median(ts)
    print(f"{model} q={n} input_tokens={tok} median_s={m:.2f} min={min(ts):.2f} max={max(ts):.2f} prompt_tok/s={tok/m:.0f}",flush=True)

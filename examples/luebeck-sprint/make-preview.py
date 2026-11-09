import sys, tomllib, math, subprocess
S=sys.argv[1]
ev=tomllib.load(open('luebeck-sprint.courseplan.toml','rb'))
k=110/25.4
def px(p): return ((p[0]-1.5)*k+643.5, (p[1]+0.15)*k+455)
ctrl={str(c['code']):c for c in ev['controls']}
sp={s['id']:s for s in ev['specials']}
course=ev['courses'][0]
seq=[('S1',sp['S1']['map_mm'],None)]+[(str(cc['code']),ctrl[str(cc['code'])]['map_mm'],cc.get('number')) for cc in course['controls']]+[('F1',sp['F1']['map_mm'],None)]
R_OUTER=3.0+0.175
def box_dist(c,s,a,b,reach):
    # distance along the ray at which a box (half sizes a, b) centred on it is `reach` from the origin
    if s<1e-9 or (c>1e-9 and (reach+a)/c*s<=b): return (reach+a)/c
    if c<1e-9 or (reach+b)/s*c<=a: return (reach+b)/s
    p=a*c+b*s
    return p+math.sqrt(max(p*p-a*a-b*b+reach*reach,0.0))
P='#A626FF'; r=3.0*k; lw=0.35*k*1.4
draw=[]
pts=[px(s[1]) for s in seq]
for i in range(len(pts)-1):
    (x1,y1),(x2,y2)=pts[i],pts[i+1]
    d=math.hypot(x2-x1,y2-y1); ux,uy=(x2-x1)/d,(y2-y1)/d
    a=(3.5 if i==0 else 3.0)*k+0.8*k; b=(3.5*k if i==len(pts)-2 else r)+0.8*k
    draw.append(f"line {x1+ux*a:.1f},{y1+uy*a:.1f} {x2-ux*b:.1f},{y2-uy*b:.1f}")
(x,y),(x2,y2)=pts[0],pts[1]; ang=math.atan2(y2-y,x2-x); R=7.0/math.sqrt(3)*k
draw.append("polygon "+" ".join(f"{x+R*math.cos(ang+t):.1f},{y+R*math.sin(ang+t):.1f}" for t in (0,2.094,4.189)))
for p in pts[1:-1]: draw.append(f"circle {p[0]:.1f},{p[1]:.1f} {p[0]+r:.1f},{p[1]:.1f}")
fx,fy=pts[-1]
for rr in (2.5,3.5): draw.append(f"circle {fx:.1f},{fy:.1f} {fx+rr*k:.1f},{fy:.1f}")
cmd=['magick',f'{S}/map.png','-fill','none','-stroke',P,'-strokewidth',f'{lw:.1f}']
for d in draw: cmd+=['-draw',d]
cmd+=['-stroke','none','-fill',P,'-font','/System/Library/Fonts/Supplemental/Arial.ttf','-pointsize','19']
for i,s in enumerate(seq[1:-1],1):
    x,y=pts[i]; n=s[2] or {'direction':45,'gap_mm':0.5}
    t=math.radians(n['direction']); c,s_=abs(math.cos(t)),abs(math.sin(t))
    # ADR 0030: gap = clearance from the outer edge of the circle line to the number box, box centred on the ray
    w=0.6*4.0*k*len(str(i)); h=4.0*k; reach=R_OUTER*k+n['gap_mm']*k
    dist=box_dist(c,s_,w/2,h/2,reach)
    cmd+=['-draw',f"text {x+dist*math.cos(t)-w/2:.0f},{y-dist*math.sin(t)+7:.0f} '{i}'"]
cmd+=[f'{S}/sketch.png']
subprocess.run(cmd,check=True)
xs=[p[0] for p in pts]; ys=[p[1] for p in pts]
x0,x1,y0,y1=int(min(xs)-60),int(max(xs)+60),int(min(ys)-60),int(max(ys)+50)
subprocess.run(['magick',f'{S}/sketch.png','-crop',f'{x1-x0}x{y1-y0}+{x0}+{y0}','+repage',f'{S}/sketch-crop.png'],check=True)

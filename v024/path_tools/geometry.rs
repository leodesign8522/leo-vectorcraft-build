//! Leo Design Path Tools: dependency-free cubic geometry foundation.
//! Not a UI tool yet. All coordinates and tolerances are document-space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct P { pub x: f64, pub y: f64 }
impl P {
    pub fn new(x:f64,y:f64)->Self { Self{x,y} }
    fn add(self,b:Self)->Self { Self::new(self.x+b.x,self.y+b.y) }
    fn sub(self,b:Self)->Self { Self::new(self.x-b.x,self.y-b.y) }
    fn mul(self,t:f64)->Self { Self::new(self.x*t,self.y*t) }
    fn lerp(self,b:Self,t:f64)->Self { self.add(b.sub(self).mul(t)) }
    fn len(self)->f64 { self.x.hypot(self.y) }
    fn finite(self)->bool { self.x.is_finite() && self.y.is_finite() }
    fn dot(self,b:Self)->f64 { self.x*b.x+self.y*b.y }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cubic(pub [P;4]);
impl Cubic {
    pub fn finite(self)->bool { self.0.iter().all(|p|p.finite()) }
    pub fn eval(self,t:f64)->P {
        let [a,b,c,d]=self.0;
        a.lerp(b,t).lerp(b.lerp(c,t),t).lerp(b.lerp(c,t).lerp(c.lerp(d,t),t),t)
    }
    pub fn derivative(self,t:f64)->P {
        let [a,b,c,d]=self.0;
        b.sub(a).lerp(c.sub(b),t).lerp(c.sub(b).lerp(d.sub(c),t),t).mul(3.0)
    }
    pub fn split(self,t:f64)->Option<(Self,Self)> {
        if !self.finite() || !t.is_finite() || !(0.0..=1.0).contains(&t) { return None; }
        let [a,b,c,d]=self.0;
        let ab=a.lerp(b,t); let bc=b.lerp(c,t); let cd=c.lerp(d,t);
        let abc=ab.lerp(bc,t); let bcd=bc.lerp(cd,t); let mid=abc.lerp(bcd,t);
        let l=Self([a,ab,abc,mid]); let r=Self([mid,bcd,cd,d]);
        (l.finite() && r.finite()).then_some((l,r))
    }
    /// Exact polynomial restriction, including extrapolation beyond endpoints.
    pub fn interval(self,a:f64,b:f64)->Option<Self> {
        if !self.finite() || !a.is_finite() || !b.is_finite() || a>=b {return None;}
        let p=self.eval(a); let q=self.eval(b); let s=(b-a)/3.0;
        let result=Self([p,p.add(self.derivative(a).mul(s)),q.sub(self.derivative(b).mul(s)),q]);
        result.finite().then_some(result)
    }
}
/// Rotate and set length relative to the anchor; preserve zero-handle semantics.
pub fn set_handle(anchor:P,handle:P,angle_delta:f64,length:Option<f64>)->Option<P>{
    if !anchor.finite() || !handle.finite() || !angle_delta.is_finite(){return None;}
    let d=handle.sub(anchor); let n=d.len();
    let l=length.unwrap_or(n);
    if !l.is_finite() || l<0.0 {return None;}
    if l==0.0 {return Some(anchor);}
    if !n.is_finite() || n<=1e-12{return None;}
    let angle=d.y.atan2(d.x)+angle_delta;
    let p=anchor.add(P::new(angle.cos()*l,angle.sin()*l));
    p.finite().then_some(p)
}
/// Straight extension follows the first nonzero terminal control direction.
pub fn extend_tangent(c:Cubic,length:f64)->Option<Cubic>{
    if !c.finite() || !length.is_finite() || length<=0.0{return None;}
    let end=c.0[3];
    let d=[c.0[2],c.0[1],c.0[0]].into_iter().find_map(|p|{
        let v=end.sub(p); let n=v.len(); (n.is_finite() && n>1e-12).then(||v.mul(1.0/n))
    })?;
    let q=end.add(d.mul(length));
    let result=Cubic([end,end.lerp(q,1.0/3.0),end.lerp(q,2.0/3.0),q]);
    result.finite().then_some(result)
}
/// Join two subdivisions of the same cubic, only if their control-point
/// discrepancy is <= tolerance. Convex-hull bound gives a continuous,
/// parameter-matched geometric error bound, not merely sampled error.
/// At a cusp or degenerate join we conservatively decline.
pub fn try_join(left:Cubic,right:Cubic,tolerance:f64)->Option<(Cubic,f64)>{
    if !left.finite() || !right.finite() || !tolerance.is_finite() || tolerance<0.0{return None;}
    if left.0[3].sub(right.0[0]).len()>tolerance{return None;}
    let x=left.0[3].sub(left.0[2]); let y=right.0[1].sub(right.0[0]);
    let nx=x.len();let ny=y.len();
    if !nx.is_finite() || !ny.is_finite() || nx<=1e-12 || ny<=1e-12 || x.dot(y)<=0.0{return None;}
    let t=nx/(nx+ny);
    if !(1e-8..1.0-1e-8).contains(&t){return None;}
    let a=left.0[0];let d=right.0[3];
    let c=Cubic([a,a.add(left.0[1].sub(a).mul(1.0/t)),d.add(right.0[2].sub(d).mul(1.0/(1.0-t))),d]);
    let (l,r)=c.split(t)?;
    let error= l.0.iter().zip(left.0).chain(r.0.iter().zip(right.0)).map(|(p,q)|p.sub(q).len()).fold(0.0,f64::max);
    (error.is_finite() && error<=tolerance).then_some((c,error))
}
/// Relocate a subdivision anchor on its original cubic without changing shape.
pub fn slide_join(left:Cubic,right:Cubic,new_t:f64,tolerance:f64)->Option<(Cubic,Cubic)>{
    if !(1e-8..1.0-1e-8).contains(&new_t){return None;}
    let (c,_)=try_join(left,right,tolerance)?;
    c.split(new_t)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn p(x:f64,y:f64)->P{P::new(x,y)}
    fn curve()->Cubic{Cubic([p(0.,0.),p(30.,80.),p(90.,-20.),p(120.,30.)])}
    fn near(a:P,b:P){assert!(a.sub(b).len()<1e-8,"{a:?} != {b:?}");}
    #[test] fn split_preserves_shape(){
        let c=curve();let(l,r)=c.split(0.37).unwrap();
        for i in 0..101{let t=i as f64/100.;near(l.eval(t),c.eval(t*0.37));near(r.eval(t),c.eval(0.37+0.63*t));}
    }
    #[test] fn join_and_slide_preserve_shape(){
        let c=curve();let(l,r)=c.split(0.37).unwrap();
        let(j,e)=try_join(l,r,1e-8).unwrap();assert!(e<1e-8);
        for i in 0..4{near(j.0[i],c.0[i]);}
        let(l,r)=slide_join(l,r,0.7,1e-8).unwrap();near(l.0[3],c.eval(0.7));near(r.0[0],c.eval(0.7));
        for i in 0..101{let t=i as f64/100.;near(l.eval(t),c.eval(t*0.7));near(r.eval(t),c.eval(0.7+0.3*t));}
    }
    #[test] fn reject_unrelated_curves_and_degenerate_input(){
        let(l,mut r)=curve().split(0.37).unwrap();r.0[2]=r.0[2].add(p(0.,20.));
        assert!(try_join(l,r,0.01).is_none());assert!(curve().split(f64::NAN).is_none());
        assert!(extend_tangent(Cubic([p(1.,1.);4]),10.).is_none());
        assert!(slide_join(l,r,0.,0.01).is_none());
    }
    #[test] fn extrapolation_matches_polynomial(){
        let c=curve();let r=c.interval(1.,1.3).unwrap();
        for i in 0..101{let t=i as f64/100.;near(r.eval(t),c.eval(1.+0.3*t));}
        near(r.0[0],c.0[3]);
    }
    #[test] fn tangent_length_and_handle_rotation(){
        let c=curve();let r=extend_tangent(c,20.).unwrap();
        near(r.0[0],c.0[3]);assert!((r.0[3].sub(r.0[0]).len()-20.).abs()<1e-8);
        near(set_handle(p(1.,2.),p(3.,2.),std::f64::consts::FRAC_PI_2,Some(5.)).unwrap(),p(1.,7.));
    }
}

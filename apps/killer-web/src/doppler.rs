use glam::Vec3;

pub fn pitch(listener: [f32;3],position: [f32;3],velocity: [f32;3],factor: f32) -> Result<f32,&'static str> {
    let direction=Vec3::from(listener)-Vec3::from(position);
    let length=direction.length();
    if length==0.0 || factor==0.0 {return Ok(1.0);}
    let speed=343.3_f32;
    let projected=direction.dot(Vec3::from(velocity))/length;
    let projected=projected.min(speed/factor);
    let result=speed/(speed-factor*projected);
    if result.is_finite() && result>0.0 {Ok(result)} else {Err("frequência Doppler fora do domínio do Web Audio")}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn openal_doppler_projection_keeps_direction_disabled_and_stationary_cases() {
        assert_eq!(pitch([0.0;3],[1.0,0.0,0.0],[0.0;3],1.0).unwrap(),1.0);
        assert_eq!(pitch([0.0;3],[1.0,0.0,0.0],[0.0,9.0,0.0],1.0).unwrap(),1.0);
        assert_eq!(pitch([0.0;3],[1.0,0.0,0.0],[7.0,0.0,0.0],0.0).unwrap(),1.0);
        assert_eq!(pitch([0.0;3],[0.0;3],[7.0,0.0,0.0],1.0).unwrap(),1.0);
        assert!(pitch([0.0;3],[1.0,0.0,0.0],[-7.0,0.0,0.0],1.0).unwrap()>1.0);
        assert!(pitch([0.0;3],[1.0,0.0,0.0],[7.0,0.0,0.0],1.0).unwrap()<1.0);
        assert!(pitch([0.0;3],[1.0,0.0,0.0],[-400.0,0.0,0.0],1.0).is_err());
    }
}

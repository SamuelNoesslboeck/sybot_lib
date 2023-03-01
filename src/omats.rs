use glam::Vec3;
use stepper_lib::{Gamma, Phi, force_gammas_from_phis, force_phis_from_gammas, Inertia, Force};

use crate::{Robot, Vectors, ConfRobot};

pub type Syomat = crate::BasicRobot<3, 1, 0>;

impl Robot<3, 0> for Syomat 
{
    type Error = std::io::Error;

    // Position
        #[inline]
        fn gammas_from_phis(&self, phis : [Phi; 3]) -> [Gamma; 3] {
            force_gammas_from_phis(phis)
        }

        #[inline]
        fn phis_from_gammas(&self, gammas : [Gamma; 3]) -> [Phi; 3] {
            force_phis_from_gammas(gammas)
        }

        #[inline]
        fn deco_axis(&self) -> Vec3 {
            self.get_tool().unwrap().get_vec()
        }
    //

    fn vecs_from_phis(&self, phis : &[Phi; 3]) -> Vectors<3> {
        let [ p_x, p_y, p_z ] = phis;
        
        [ Vec3::X * p_x.0, Vec3::Y * p_y.0, Vec3::Z * p_z.0 ]
    }

    fn phis_from_def_vec(&self, pos : Vec3) -> [Phi; 3] {
        [ Phi(pos.x), Phi(pos.y), Phi(pos.z) ]
    }

    fn reduce_to_def(&self, pos : Vec3, dec_ang : [f32; 0]) -> Vec3 {
        todo!()
    }

    fn phis_from_vec(&self, pos : Vec3, dec_ang : [f32; 0]) -> [Phi; 3] {
        todo!()
    }

    fn inertias_from_vecs(&self, vecs : &Vectors<3>) -> [Inertia; 3] {
        todo!()
    }

    fn forces_from_vecs(&self, vecs : &Vectors<3>) -> [Force; 3] {
        todo!()
    }

    fn update(&mut self, phis : Option<&[Phi; 3]>) {
        todo!()
    }

    fn measure(&mut self, acc : u64) -> Result<(), [bool; 3]> {
        todo!()
    }

    fn measure_async(&mut self, acc : u64) {
        todo!()
    }

    fn set_limit(&mut self) {
        todo!()
    }
}
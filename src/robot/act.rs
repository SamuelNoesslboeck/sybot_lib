use glam::Vec3;
use stepper_lib::units::*;

use crate::{Robot, Points, Vectors};

pub trait ActRobot<const COMP : usize, const DECO : usize, const DIM : usize, const ROT : usize> 
    : Robot<COMP, DECO, DIM, ROT>
{
    // Types
        type Error : std::error::Error;
    // 

    // Position
        /// Returns all the angles used by the controls to represent the components extension/drive distance
        #[inline]
        fn gammas(&self) -> [Gamma; COMP] {
            self.comps().gammas()
        }

        /// Converts all angles (by subtracting an offset in most of the cases)
        fn gammas_from_phis(&self, phis : [Phi; COMP]) -> [Gamma; COMP];

        #[inline]
        fn phis(&self) -> [Phi; COMP] {
            self.phis_from_gammas(self.gammas())
        }

        fn phis_from_gammas(&self, gammas : [Gamma; COMP]) -> [Phi; COMP];

        // Other
            fn deco_axis(&self) -> Vec3;
        //
    //

    // Calculation
        // Position
            #[inline]
            fn vecs_from_gammas(&self, gammas : &[Gamma; COMP]) -> Vectors<COMP> {
                self.vecs_from_phis(&self.phis_from_gammas(*gammas))
            }

            #[inline]
            fn points_from_gammas(&self, gammas : &[Gamma; COMP]) -> Points<COMP> {
                self.points_from_vecs(&self.vecs_from_gammas(gammas))
            }
            
            fn vecs_from_phis(&self, phis : &[Phi; COMP]) -> Vectors<COMP>;

            #[inline]
            fn points_from_phis(&self, phis : &[Phi; COMP]) -> Points<COMP> {
                self.points_from_vecs(&self.vecs_from_phis(phis))
            }

            #[inline]
            fn gammas_from_def_vec(&self, pos : Vec3) -> [Gamma; COMP] {
                self.gammas_from_phis(self.phis_from_def_vec(pos))
            }

            fn phis_from_def_vec(&self, pos : Vec3) -> [Phi; COMP];

            fn points_from_vecs(&self, vecs : &Vectors<COMP>) -> Points<COMP> {
                let mut points : Points<COMP> = [Vec3::ZERO; COMP];
                for i in 0 .. COMP {
                    points[i] += *self.anchor();
                    for n in 0 .. (i + 1) {
                        points[i] += vecs[n];
                    }
                }

                *points.last_mut().unwrap() += self.get_tool().unwrap().get_vec();
                points
            }

            fn vecs_from_points(&self, points : &Vectors<COMP>) -> Vectors<COMP> {
                let mut vecs : Points<COMP> = [Vec3::ZERO; COMP];
                vecs[0] = points[0] - *self.anchor();
                for i in 1 .. COMP {
                    vecs[i] = points[i] - points[i - 1];
                }
                vecs
            }

            fn reduce_to_def(&self, pos : Vec3, deco : [f32; DECO]) -> Vec3;

            fn phis_from_vec(&self, pos : Vec3, deco : [f32; DECO]) -> [Phi; COMP];

            // Current
            fn pos(&self) -> Vec3 {
                *self.points_from_phis(&self.phis()).last().unwrap()
            }
        //

        // Load
            #[inline]
            fn inertias_from_phis(&self, phis : &[Phi; COMP]) -> [Inertia; COMP] {
                self.inertias_from_vecs(&self.vecs_from_phis(phis))
            }

            #[inline]
            fn forces_from_phis(&self, phis : &[Phi; COMP]) -> [Force; COMP] {
                self.forces_from_vecs(&self.vecs_from_phis(phis))
            }

            fn inertias_from_vecs(&self, vecs : &Vectors<COMP>) -> [Inertia; COMP];

            fn forces_from_vecs(&self, vecs : &Vectors<COMP>) -> [Force; COMP];
        // 

        fn update(&mut self, phis : Option<&[Phi; COMP]>) -> Result<(), crate::Error>;
    //

    // Writing values
        #[inline]
        fn apply_inertias(&mut self, inertias : &[Inertia; COMP]) {
            self.comps_mut().apply_inertias(inertias);
        }

        #[inline]
        fn apply_forces(&mut self, forces : &[Force; COMP]) {
            self.comps_mut().apply_forces(forces);
        }

        // Position
            #[inline]
            fn write_gammas(&mut self, gammas : &[Gamma; COMP]) {
                self.comps_mut().write_gammas(gammas);
            }

            fn write_phis(&mut self, phis : &[Phi; COMP]) {
                let gammas = self.gammas_from_phis(*phis);
                self.comps_mut().write_gammas(&gammas)
            }
        // 
    // 

    // Movement
        #[inline]
        fn drive_rel(&mut self, deltas : [Delta; COMP]) -> Result<[Delta; COMP], stepper_lib::Error> {
            let vels = *self.max_vels();
            self.comps_mut().drive_rel(deltas, vels)
        }

        #[inline]
        fn drive_abs(&mut self, gammas : [Gamma; COMP]) -> Result<[Delta; COMP], stepper_lib::Error> {
            let vels = *self.max_vels();
            self.comps_mut().drive_abs(gammas, vels)
        }

        // Async 
        #[inline]
        fn drive_rel_async(&mut self, deltas : [Delta; COMP]) -> Result<(), stepper_lib::Error> {
            let vels = *self.max_vels();
            self.comps_mut().drive_rel_async(deltas, vels)
        }
        
        #[inline]
        fn drive_abs_async(&mut self, gammas : [Gamma; COMP]) -> Result<(), stepper_lib::Error> {
            let vels = *self.max_vels();
            self.comps_mut().drive_abs_async(gammas, vels)
        }

        // Single Component
            #[inline]
            fn drive_comp_rel(&mut self, index : usize, delta : Delta) -> Result<Delta, stepper_lib::Error> {
                let vels = *self.max_vels();
                self.comps_mut()[index].drive_rel(delta, vels[index])
            }

            #[inline]
            fn drive_comp_abs(&mut self, index : usize, gamma : Gamma) -> Result<Delta, stepper_lib::Error> {
                let vels = *self.max_vels();
                self.comps_mut()[index].drive_abs(gamma, vels[index])
            }

            #[inline]
            fn drive_comp_rel_async(&mut self, index : usize, delta : Delta) -> Result<(), stepper_lib::Error> {
                let vels = *self.max_vels();
                self.comps_mut()[index].drive_rel_async(delta, vels[index])
            }

            #[inline]
            fn drive_comp_abs_async(&mut self, index : usize, gamma : Gamma) -> Result<(), stepper_lib::Error> {
                let vels = *self.max_vels();
                self.comps_mut()[index].drive_abs_async(gamma, vels[index])
            }
        //

        // Measure
            fn measure(&mut self) -> Result<[Delta; 4], stepper_lib::Error>;
        // 

        #[inline]
        fn await_inactive(&mut self) -> Result<[Delta; COMP], stepper_lib::Error> {
            self.comps_mut().await_inactive()
        }

        #[inline]
        fn set_end(&mut self, gammas : &[Gamma; COMP]) {
            self.comps_mut().set_ends(gammas)
        }

        fn set_limit(&mut self) {
            for i in 0 .. 4 {
                let min = self.mach().limit[i].min;
                let max = self.mach().limit[i].max;

                self.comps_mut()[i].set_limit(min, max);
            }
        }
    // 
}
/*
MIT License
Copyright (c) 2021 Germán Molina
Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:
The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.
THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/
use crate::construction::Discretization;
use crate::Float;
use calendar::Date;

use communication_protocols::{ErrorHandling, MetaOptions, SimulationModel};
use geometry3d::Vector3D;
use weather::Weather;

use crate::surface::{SurfaceTrait, ThermalFenestration, ThermalSurface, ThermalSurfaceData};

use crate::heating_cooling::calc_cooling_heating_power;

use crate::zone::ThermalZone;
use simple_model::{Boundary, SimpleModel, SimulationState, SimulationStateHeader};

pub struct ThermalModel {
    /// All the Thermal Zones in the model
    pub zones: Vec<ThermalZone>,

    /// All the surfaces in the model
    pub surfaces: Vec<ThermalSurface>,

    /// All the Fenestrations in the model
    pub fenestrations: Vec<ThermalFenestration>,

    /// The number of steps that this model needs
    /// to take in order to advance one step of the main
    /// simulation.
    pub dt_subdivisions: usize,

    /// The model's dt (i.e., main_dt / self.dt_subdivisions)
    pub dt: Float,
}

impl ErrorHandling for ThermalModel {
    fn module_name() -> &'static str {
        "Thermal model"
    }
}

impl SimulationModel for ThermalModel {
    type Type = Self;
    type OptionType = (); // No options

    /// Creates a new ThermalModel from a SimpleModel.
    ///    
    /// # Inputs:
    /// * model: the `SimpleModel` that the model represents
    /// * state: the `SimulationStateHeader` attached to the SimpleModel
    /// * n: the number of timesteps per hour taken by the main simulation.
    fn new(
        _meta_options: &MetaOptions,
        _options: (),
        model: &SimpleModel,
        state: &mut SimulationStateHeader,
        n: usize,
    ) -> Result<Self, String> {
        /* CREATE ALL ZONES, ONE PER SPACE */
        let mut thermal_zones: Vec<ThermalZone> = Vec::with_capacity(model.spaces.len());
        for (i, space) in model.spaces.iter().enumerate() {
            // Add the zone to the model... this pushes it to the sate
            // as well
            thermal_zones.push(ThermalZone::from_space(space, state, i));
        }

        /* CREATE ALL SURFACES AND FENESTRATIONS, AND IDENTIFY MODEL TIMESTEP  */

        // choose the smallest timestep in all constructions

        let max_dx = 0.04; // 4cm
        let min_dt = 60.; // 60 seconds

        let mut n_subdivisions: usize = 1;
        let main_dt = 60. * 60. / n as Float;

        // Store the dts and n_nodes somwehere. Take note of the largest
        // number of subditivions required
        let mut thermal_surfaces = Vec::with_capacity(model.surfaces.len());
        for (i, surf) in model.surfaces.iter().enumerate() {
            let constr = &surf.construction;

            let normal = surf.vertices.normal();
            let cos_tilt = normal * Vector3D::new(0., 0., 1.);
            #[cfg(debug_assertions)]
            dbg!("height is 1");
            let height = 1.;
            let angle = cos_tilt.acos();
            let area = surf.area();
            let perimeter = surf.vertices.outer().perimeter().unwrap();

            let d = Discretization::new(constr, main_dt, max_dx, min_dt, height, angle)?;

            if d.tstep_subdivision > n_subdivisions {
                n_subdivisions = d.tstep_subdivision;
            }
            let mut tsurf =
                ThermalSurface::new(state, i, surf, area, perimeter, normal, constr, d)?;
            // Match surface and zones
            if let Ok(b) = surf.front_boundary() {
                tsurf.set_front_boundary(b.clone());
            }
            if let Ok(b) = surf.back_boundary() {
                tsurf.set_back_boundary(b.clone());
            }

            thermal_surfaces.push(tsurf);
        }

        let mut thermal_fens = Vec::with_capacity(model.fenestrations.len());
        for (i, surf) in model.fenestrations.iter().enumerate() {
            let constr = &surf.construction;

            let normal = surf.vertices.normal();
            let cos_tilt = normal * Vector3D::new(0., 0., 1.);
            let angle = cos_tilt.acos();
            let area = surf.area();
            let perimeter = surf.vertices.outer().perimeter().unwrap();

            #[cfg(debug_assertions)]
            dbg!("height is 1");
            let height = 1.;

            let d = Discretization::new(constr, main_dt, max_dx, min_dt, height, angle)?;

            if d.tstep_subdivision > n_subdivisions {
                n_subdivisions = d.tstep_subdivision;
            }
            let mut tsurf =
                ThermalFenestration::new(state, i, surf, area, perimeter, normal, constr, d)?;
            // Match surface and zones
            if let Ok(b) = surf.front_boundary() {
                tsurf.set_front_boundary(b.clone());
            }
            if let Ok(b) = surf.back_boundary() {
                tsurf.set_back_boundary(b.clone());
            }
            thermal_fens.push(tsurf);
        }

        // This is the model's dt now. When marching
        let mut dt = 60. * 60. / (n as Float * n_subdivisions as Float);

        // safety.
        const SAFETY: usize = 1;
        dt /= SAFETY as Float;
        n_subdivisions *= SAFETY;

        Ok(ThermalModel {
            zones: thermal_zones,
            surfaces: thermal_surfaces,
            fenestrations: thermal_fens,
            dt_subdivisions: n_subdivisions,
            dt,
        })
    }

    /// Advances one main_timestep through time. That is,
    /// it performs `self.dt_subdivisions` steps, advancing
    /// `self.dt` seconds in each of them.
    fn march(
        &self,
        mut date: Date,
        weather: &dyn Weather,
        model: &SimpleModel,
        state: &mut SimulationState,
    ) -> Result<(), String> {
        // Iterate through all the sub-subdivitions
        for _ in 0..self.dt_subdivisions {
            // advance in time
            date.add_seconds(self.dt);
            let current_weather = weather.get_weather_data(date);
            let wind_direction = current_weather.wind_direction.unwrap().to_radians();
            let wind_speed = current_weather.wind_speed.unwrap();

            let t_out = match current_weather.dry_bulb_temperature {
                Some(v) => v,
                None => return Err(
                    "Trying to march on Thermal Model, but dry bulb temperature was not provided"
                        .to_string(),
                ),
            };

            let t_current = self.get_current_zones_temperatures(state);

            /* UPDATE SURFACE'S TEMPERATURES */
            for i in 0..self.surfaces.len() {
                // get surface
                let s = &self.surfaces[i];

                // find t_in and t_out of surface.
                let t_front = match &s.front_boundary {
                    Some(b) => match b {
                        Boundary::Space(space) => t_current[*space.index().unwrap()],
                        Boundary::Ground => unimplemented!(),
                    },
                    None => t_out,
                };
                let t_back = match &s.back_boundary {
                    Some(b) => match b {
                        Boundary::Space(space) => t_current[*space.index().unwrap()], //self.zones[z_index].temperature(model, state),
                        Boundary::Ground => unimplemented!(),
                    },
                    None => t_out,
                };

                // Update temperatures
                let (q_front, q_back) =
                    s.march(state, t_front, t_back, wind_direction, wind_speed, self.dt);
                model.surfaces[i].set_front_convective_heat_flow(state, q_front);
                model.surfaces[i].set_back_convective_heat_flow(state, q_back);
            } // end of iterating surface

            // What  if they are open???
            for i in 0..self.fenestrations.len() {
                // get surface
                let s = &self.fenestrations[i];

                // find t_in and t_out of surface.
                let t_front = match &s.front_boundary {
                    Some(b) => match b {
                        Boundary::Space(space) => t_current[*space.index().unwrap()],
                        Boundary::Ground => unimplemented!(),
                    },
                    None => t_out,
                };
                let t_back = match &s.back_boundary {
                    Some(b) => match b {
                        Boundary::Space(space) => t_current[*space.index().unwrap()],
                        Boundary::Ground => unimplemented!(),
                    },
                    None => t_out,
                };

                // Update temperatures
                let (q_front, q_back) =
                    s.march(state, t_front, t_back, wind_direction, wind_speed, self.dt);
                model.fenestrations[i].set_front_convective_heat_flow(state, q_front);
                model.fenestrations[i].set_back_convective_heat_flow(state, q_back);
            } // end of iterating surface

            /* UPDATE ZONES' TEMPERATURE */
            // This is done analytically.
            let (a, b, c) = self.calculate_zones_abc(model, state);

            let future_temperatures =
                self.estimate_zones_future_temperatures(&t_current, &a, &b, &c, self.dt);
            for (i, zone) in self.zones.iter().enumerate() {
                debug_assert!(!future_temperatures[i].is_nan());
                zone.reference_space
                    .set_dry_bulb_temperature(state, future_temperatures[i]);
            }
        } // End of 'in each sub-timestep-subdivision'

        Ok(())
    }
}

impl ThermalModel {
    /// Retrieves the dt_subdivisions (i.e. the
    /// number of substimesteps per timestep of this
    /// model)
    pub fn dt_subdivisions(&self) -> usize {
        self.dt_subdivisions
    }

    /// Retrieves a ThermalZone
    pub fn get_thermal_zone(&self, index: usize) -> Result<&ThermalZone, String> {
        if index >= self.zones.len() {
            return ThermalModel::internal_error(format!(
                "Ouf of bounds: Thermal Zone number {} does not exist",
                index
            ));
        }

        Ok(&self.zones[index])
    }

    // /// Retrieves a ThermalSurface
    // pub fn get_thermal_surface(&self, index: usize) -> Result<&ThermalSurface, String> {
    //     if index >= self.surfaces.len() {
    //         return ThermalModel::internal_error(format!(
    //             "Ouf of bounds: Thermal Surface number {} does not exist",
    //             index
    //         ));
    //     }

    //     Ok(&self.surfaces[index])
    // }

    // /// Retrieves a THermalFenestration
    // pub fn get_thermal_fenestration(&self, index: usize) -> Result<&ThermalSurface, String> {
    //     if index >= self.fenestrations.len() {
    //         return ThermalModel::internal_error(format!(
    //             "Ouf of bounds: Thermal Surface number {} does not exist",
    //             index
    //         ));
    //     }

    //     Ok(&self.fenestrations[index])
    // }

    /// This estimation assumes nothing changes during this time.
    /// This is self evidently wrong, as we know that, for example, the surface temperatures
    /// will change together with the zone air temperature. However, in short periods of time
    /// this can actually work.
    ///
    /// Everything starts from the following equation, representing a heat balance over
    /// the air and the contents of the Thermal zone.
    ///
    /// ```math
    /// C_{zone}\frac{dT(t)}{dt} = \displaystyle\sum_{i=loads}{Q_i} + \displaystyle\sum_{i=surf.}{h_iA_i(T_i-T)}+\displaystyle\sum_{i=otherzones}{\dot{m_i}C_p(T_i-T)}+\dot{m}_{inf}C_p(T_{out}-T)+\dot{m}_{supplied}C_p(T_{sup}-T)
    /// ```
    /// Which can be expanded into the following
    ///
    /// ```math
    /// C_{zone}\frac{dT(t)}{dt} = A - B T
    /// ```
    ///
    /// Where $`A`$ and $`B`$ are constant terms (at least they can be assumed to be during this brief period of time).
    ///
    /// ```math
    /// A = \displaystyle\sum_{i=loads}{Q_i} + \displaystyle\sum_{i=surf.}{h_iA_i T_i}+\displaystyle\sum_{i=otherzones}{\dot{m_i}C_pT_i}+\dot{m}_{inf}C_pT_{out}+\dot{m}_{supplied}C_pT_{sup}
    /// ```
    ///
    /// ```math
    /// B= \displaystyle\sum_{i=surf.}{h_iA_i}+\displaystyle\sum_{i=otherzones}{\dot{m_i}C_p}+\dot{m}_{inf}C_p +\dot{m}_{supplied}C_p
    /// ```
    ///
    /// And so, (solving the differential equation) the Temperature $`T`$ at a time $`t`$ into the future
    /// can be estimated based on the current Temperature of the zone ($`T_{current}`$) and the following
    /// equation:
    ///
    /// ```math
    ///  T(t) = \frac{A}{B} + \left( T_{current} - \frac{A}{B} \right)e^{-\frac{B}{C_{zone}}t}
    /// ```
    ///
    /// And the average temperature during the same periood is:
    /// ```math
    /// \frac{\displaystyle\int_{0}^t{T(t)dt}}{t} = \frac{A}{B}+\frac{C_{zone}\left(T_{current}-\frac{A}{B}\right)}{Bt}\left(1-e^{-\frac{Bt}{C_{zone}}} \right)
    /// ```
    fn calculate_zones_abc(
        &self,
        model: &SimpleModel,
        state: &SimulationState,
    ) -> (Vec<Float>, Vec<Float>, Vec<Float>) {
        let nzones = self.zones.len();
        // Initialize vectors containing a and b
        let mut a = vec![0.0; nzones];
        let mut b = vec![0.0; nzones];
        let mut c = vec![0.0; nzones];

        /* Qi */
        // Heating/Cooling
        for hvac in model.hvacs.iter() {
            for (target_space_index, heating_cooling) in calc_cooling_heating_power(hvac, state) {
                a[target_space_index] += heating_cooling;
            }
            // heating through air supply?
        }
        // Heating/Cooling
        for luminaire in model.luminaires.iter() {
            if let Ok(target_space) = luminaire.target_space() {
                let target_space_index = *target_space.index().unwrap();
                let consumption = luminaire
                    .power_consumption(state)
                    .expect("Luminaire has no Power Consumption state");
                a[target_space_index] += consumption;
            }
        }

        let air = crate::gas::Gas::air();
        // Other
        for (i, zone) in self.zones.iter().enumerate() {
            let space = &model.spaces[i];
            /* INFILTRATION AND VENTILATION */
            // infiltration from outside
            if let Some(t_inf_inwards) = space.infiltration_temperature(state) {
                let v_inf = space
                    .infiltration_volume(state)
                    .expect("Space has infiltration temperature but not volume");

                let cp_inf_inwards = air.heat_capacity(t_inf_inwards + 273.15);
                let rho_inf_inwards = air.density(t_inf_inwards + 273.15);
                a[i] += rho_inf_inwards * v_inf * cp_inf_inwards * t_inf_inwards;
                b[i] += rho_inf_inwards * v_inf * cp_inf_inwards;
            }

            // ventilation
            if let Some(t_vent_inwards) = space.ventilation_temperature(state) {
                let v_vent = space
                    .ventilation_volume(state)
                    .expect("Space has ventilation temperature but not volume");
                let cp_vent_inwards = air.heat_capacity(t_vent_inwards + 273.15);
                let rho_vent_inwards = air.density(t_vent_inwards + 273.15);
                a[i] += rho_vent_inwards * v_vent * cp_vent_inwards * t_vent_inwards;
                b[i] += rho_vent_inwards * v_vent * cp_vent_inwards;
            }

            // Mixing with other zones

            /* CAPACITANCE */
            let temp = space
                .dry_bulb_temperature(state)
                .expect("Zone has no Temperature!");
            c[i] = zone.mcp(temp);
        }

        /* SURFACES */
        fn iterate_surfaces<T: SurfaceTrait>(
            surfaces: &[ThermalSurfaceData<T>],
            state: &SimulationState,
            a: &mut [Float],
            b: &mut [Float],
        ) {
            for surface in surfaces {
                let parent = &surface.parent;
                let h_front = parent.front_convection_coefficient(state).unwrap();
                let h_back = parent.back_convection_coefficient(state).unwrap();

                let ai = surface.area;
                // if front leads to a Zone
                if let Some(Boundary::Space(space)) = &surface.front_boundary {
                    let z_index = space.index().unwrap();

                    let temp = surface.parent.front_temperature(state);

                    a[*z_index] += h_front * ai * temp;
                    b[*z_index] += h_front * ai;
                }

                // if back leads to a Zone
                if let Some(Boundary::Space(space)) = &surface.back_boundary {
                    let z_index = space.index().unwrap();

                    let temp = surface.parent.back_temperature(state);
                    a[*z_index] += h_back * ai * temp;
                    b[*z_index] += h_back * ai;
                }
            }
        }

        iterate_surfaces(&self.surfaces, state, &mut a, &mut b);
        iterate_surfaces(&self.fenestrations, state, &mut a, &mut b);

        /* AIR MIXTURE WITH OTHER ZONES */
        // unimplemented();

        // RETURN
        (a, b, c)
    }

    /// Retrieves a vector of the current temperatures of all the Zones as
    /// registered in the Simulation State
    fn get_current_zones_temperatures(&self, state: &SimulationState) -> Vec<Float> {
        let nzones = self.zones.len();
        // Initialize return
        let mut ret: Vec<Float> = Vec::with_capacity(nzones);
        for zone in self.zones.iter() {
            let t_current = zone.reference_space.dry_bulb_temperature(state).unwrap();
            ret.push(t_current);
        }
        ret
    }

    /// Uses an analytical solution to estimate an average temperature for each Zone
    /// for the near future. Uses the coefficients $`A`$, $`B`$ and $`C`$
    /// calculated by `calculate_zones_abc` and the Zones' current temperatures
    /// `t_current` as calculated by `get_current_temperatures`.
    #[allow(dead_code)]
    fn estimate_zones_mean_future_temperatures(
        &self,
        t_current: &[Float],
        a: &[Float],
        b: &[Float],
        c: &[Float],
        future_time: Float,
    ) -> Vec<Float> {
        let nzones = self.zones.len();
        // Initialize return
        let mut ret: Vec<Float> = Vec::with_capacity(nzones);

        for i in 0..self.zones.len() {
            let current_temp = t_current[i];
            if b[i].abs() > 1e-9 {
                // is this an apropriate threshold?
                ret.push(
                    a[i] / b[i]
                        + (c[i] * (current_temp - a[i] / b[i]) / future_time / b[i])
                            * (1.0 - (-b[i] * future_time / c[i]).exp()),
                );
            } else {
                ret.push(current_temp);
            }
        }

        ret
    }

    /// Uses an analytical solution to estimate the future Zones temperature
    /// for the near future. Uses the coefficients $`A`$, $`B`$ and $`C`$
    /// calculated by `calculate_zones_abc` and the Zones' current temperatures
    /// `t_current` as calculated by `get_current_temperatures`.
    fn estimate_zones_future_temperatures(
        &self,
        t_current: &[Float],
        a: &[Float],
        b: &[Float],
        c: &[Float],
        future_time: Float,
    ) -> Vec<Float> {
        let nzones = self.zones.len();
        // Initialize return
        let mut ret: Vec<Float> = Vec::with_capacity(nzones);
        for i in 0..nzones {
            if b[i].abs() > 1e-9 {
                // is this an apropriate threshold?
                ret.push(
                    a[i] / b[i] + (t_current[i] - a[i] / b[i]) * (-b[i] * future_time / c[i]).exp(),
                );
            } else {
                // A space that is disconnected from everything... maintains its temperature
                ret.push(t_current[i]);
            }
        }

        ret
    }
}

/***********/
/* TESTING */
/***********/

#[cfg(test)]
mod testing {
    use super::*;
    // use crate::construction::*;

    use simple_test_models::*;

    const META_OPTIONS: MetaOptions = MetaOptions {
        latitude: 0.,
        longitude: 0.,
        standard_meridian: 0.,
    };

    #[test]
    fn test_calculate_zones_abc() {
        let (simple_model, mut state_header) = get_single_zone_test_building(
            // &mut state,
            &SingleZoneTestBuildingOptions {
                zone_volume: 40.,
                surface_area: 4.,
                construction: vec![TestMat::Polyurethane(0.02)],
                emmisivity: 0.0,
                ..Default::default()
            },
        );

        let n: usize = 1;
        let thermal_model =
            ThermalModel::new(&META_OPTIONS, (), &simple_model, &mut state_header, n).unwrap();
        let state = state_header.take_values().unwrap();
        // MAP THE STATE
        // model.map_simulation_state(&mut state).unwrap();

        // Test
        let (a, b, c) = thermal_model.calculate_zones_abc(&simple_model, &state);
        assert_eq!(a.len(), 1);
        assert_eq!(c.len(), 1);
        assert_eq!(b.len(), 1);
        assert_eq!(c[0], thermal_model.get_thermal_zone(0).unwrap().mcp(22.));
        let hi = simple_model.surfaces[0]
            .front_convection_coefficient(&state)
            .unwrap();

        let temp = &thermal_model.surfaces[0].parent.front_temperature(&state);
        let area = &thermal_model.surfaces[0].area;
        assert_eq!(a[0], area * hi * temp);
        assert_eq!(b[0], area * hi);
    }
}

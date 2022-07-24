use communication_protocols::SimulationModel;
use heat::model::ThermalModel;
use heat::Float;

use calendar::Date;
use communication_protocols::MetaOptions;
use schedule::ScheduleConstant;
use validate::*;
use weather::SyntheticWeather;
// use simple_model::{SimulationStateElement, HVAC};
use simple_test_models::{get_single_zone_test_building, SingleZoneTestBuildingOptions, TestMat};

const META_OPTIONS: MetaOptions = MetaOptions {
    latitude: 0.,
    longitude: 0.,
    standard_meridian: 0.,
};

fn get_validator(
    expected: Vec<f64>,
    found: Vec<f64>,
    expected_legend: &'static str,
) -> Box<SeriesValidator> {
    Box::new(SeriesValidator {
        x_label: Some("time step"),
        y_label: Some("Zone Temperature"),
        y_units: Some("C"),

        expected_legend: Some(expected_legend),
        found_legend: Some("SIMPLE"),
        expected,
        found,
        ..validate::SeriesValidator::default()
    })
}

// /// A single-zone test model with walls assumed to have
// /// no mass. It has a closed solution, which is nice.
// ///
// /// There is no sun.
// #[derive(Default)]
// struct SingleZoneTestModel {
//     /// volume of the zone (m3)
//     zone_volume: Float,

//     /// Facade area (m2)
//     surface_area: Float,

//     /// the R-value of the facade
//     facade_r: Float,

//     /// Infiltration rate (m3/s)
//     infiltration_rate: Float,

//     /// Heating power (Watts)
//     heating_power: Float,

//     /// Lighting power (Watts)
//     lighting_power: Float,

//     /// Temperature outside of the zone
//     temp_out: Float,

//     /// Temperature at the beginning
//     temp_start: Float,
// }

// impl SingleZoneTestModel {
//     fn get_closed_solution(&self) -> Box<impl Fn(Float) -> Float> {
//         // heat balance in the form
//         // of C*dT/dt = A - B*T
//         let air = heat::gas::Gas::air();
//         let rho = air.density(22. + 273.15); //kg/m3
//         let cp = air.heat_capacity(22. + 273.15); //J/kg.K
//         let u = 1. / self.facade_r;

//         let c = self.zone_volume * rho * cp;

//         let a = self.heating_power
//             + self.lighting_power
//             + self.temp_out * u * self.surface_area
//             + self.infiltration_rate * rho * cp * self.temp_out;

//         let b = u * self.surface_area + rho * self.infiltration_rate * cp;

//         let k1 = self.temp_start - a / b;

//         let f = move |t: Float| -> Float { a / b + k1 * (-b * t / c).exp() };

//         Box::new(f)
//     }
// }

// fn march_with_window() -> (Vec<Float>, Vec<Float>) {
//     let surface_area = 4.;
//     let window_area = 1.;
//     let zone_volume = 40.;

//     let (simple_model, mut state_header) = get_single_zone_test_building(
//         // &mut state,
//         &SingleZoneTestBuildingOptions {
//             zone_volume,
//             surface_area,
//             window_area,
//             construction: vec![TestMat::Polyurethane(0.02)],
//             emmisivity: 0.0,
//             ..Default::default()
//         },
//     );

//     // Finished model the SimpleModel
//     let n: usize = 6;
//     let main_dt = 60. * 60. / n as Float;
//     let thermal_model = ThermalModel::new(&META_OPTIONS, (), &simple_model, &mut state_header, n).unwrap();

//     let mut state = state_header.take_values().unwrap();

//     // MAP THE STATE
//     // model.map_simulation_state(&mut state).unwrap();

//     // START TESTING.
//     let hs_front = simple_model.surfaces[0]
//         .front_convection_coefficient(&state)
//         .unwrap();
//         let hs_back = simple_model.surfaces[0]
//         .back_convection_coefficient(&state)
//         .unwrap();

//     let r = thermal_model.surfaces[0].discretization.r_value() + 1. / hs_front + 1. / hs_back;

//     // Initial T of the zone
//     let t_start = thermal_model.zones[0]
//         .reference_space
//         .dry_bulb_temperature(&state)
//         .unwrap();

//     let t_out: Float = 30.0; // T of surroundings

//     let mut weather = SyntheticWeather::default();
//     weather.dry_bulb_temperature = Box::new(ScheduleConstant::new(t_out));
//     weather.wind_direction = Box::new(ScheduleConstant::new(0.0));
//     weather.wind_speed = Box::new(ScheduleConstant::new(0.0));

//     let dt = main_dt;

//     let mut date = Date {
//         day: 1,
//         hour: 0.0,
//         month: 1,
//     };

//     // test model
//     let tester = SingleZoneTestModel {
//         zone_volume,
//         surface_area, // the window is a hole on the wall... does not add area
//         facade_r: r,
//         temp_out: t_out,
//         temp_start: t_start,
//         ..SingleZoneTestModel::default()
//     };
//     let exp_fn = tester.get_closed_solution();

//     // March:
//     let n = 80;
//     let mut exp = Vec::with_capacity(n);
//     let mut found = Vec::with_capacity(n);
//     for i in 0..n {
//         let time = (i as Float) * dt;
//         date.add_seconds(time);

//         let found_v = thermal_model.zones[0]
//             .reference_space
//             .dry_bulb_temperature(&state)
//             .unwrap();

//         thermal_model
//             .march(date, &weather, &simple_model, &mut state)
//             .unwrap();

//         // Get exact solution.
//         let exp_v = exp_fn(time);
//         exp.push(exp_v);
//         found.push(found_v);
//     }
//     (exp, found)
// }

// fn very_simple_march() -> (Vec<Float>, Vec<Float>) {
//     let zone_volume = 40.;
//     let surface_area = 4.;
//     let (simple_model, mut state_header) = get_single_zone_test_building(
//         // &mut state,
//         &SingleZoneTestBuildingOptions {
//             zone_volume,
//             surface_area,
//             construction: vec![TestMat::Polyurethane(0.02)],
//             emmisivity: 0.0,
//             ..Default::default()
//         },
//     );

//     let n: usize = 60;
//     let main_dt = 60. * 60. / n as Float;
//     let thermal_model = ThermalModel::new(&META_OPTIONS, (),&simple_model, &mut state_header, n).unwrap();

//     let mut state = state_header.take_values().unwrap();

//     let hs_front = simple_model.surfaces[0]
//         .front_convection_coefficient(&state)
//         .unwrap();
//     let hs_back = simple_model.surfaces[0]
//         .back_convection_coefficient(&state)
//         .unwrap();

//     let r = thermal_model.surfaces[0].discretization.r_value() + 1. / hs_front + 1. / hs_back;

//     // Initial T of the zone
//     let t_start = thermal_model.zones[0]
//         .reference_space
//         .dry_bulb_temperature(&state)
//         .unwrap();

//     let t_out: Float = 30.0; // T of surroundings

//     // test model
//     let tester = SingleZoneTestModel {
//         zone_volume,
//         surface_area,
//         facade_r: r,
//         temp_out: t_out,
//         temp_start: t_start,
//         ..SingleZoneTestModel::default()
//     };
//     let exp_fn = tester.get_closed_solution();

//     let mut weather = SyntheticWeather::default();
//     weather.dry_bulb_temperature = Box::new(ScheduleConstant::new(t_out));
//     weather.wind_direction = Box::new(ScheduleConstant::new(0.0));
//     weather.wind_speed = Box::new(ScheduleConstant::new(0.0));

//     let mut date = Date {
//         day: 1,
//         hour: 0.0,
//         month: 1,
//     };

//     let n = 1000;
//     let mut exp = Vec::with_capacity(n);
//     let mut found = Vec::with_capacity(n);
//     for i in 0..1000 {
//         let time = (i as Float) * main_dt;
//         date.add_seconds(time);

//         let found_v = thermal_model.zones[0]
//             .reference_space
//             .dry_bulb_temperature(&state)
//             .unwrap();

//         thermal_model
//             .march(date, &weather, &simple_model, &mut state)
//             .unwrap();

//         // Get exact solution.
//         let exp_v = exp_fn(time);

//         exp.push(exp_v);
//         found.push(found_v);
//     }

//     return (exp, found);
// }

// fn march_with_window_and_luminaire() -> (Vec<Float>, Vec<Float>) {
//     let surface_area = 4.;
//     let zone_volume = 40.;
//     let lighting_power = 100.;

//     let (simple_model, mut state_header) = get_single_zone_test_building(
//         // &mut state,
//         &SingleZoneTestBuildingOptions {
//             zone_volume,
//             surface_area,
//             lighting_power,
//             construction: vec![TestMat::Polyurethane(0.02)],
//             emmisivity: 0.0,
//             ..Default::default()
//         },
//     );

//     // Finished model the SimpleModel

//     let n: usize = 20;
//     let main_dt = 60. * 60. / n as Float;
//     let thermal_model = ThermalModel::new(&META_OPTIONS, (),&simple_model, &mut state_header, n).unwrap();

//     let mut state = state_header.take_values().unwrap();

//     // turn the lights on
//     let lum_state_i = simple_model.luminaires[0]
//         .power_consumption_index()
//         .unwrap();
//     state[lum_state_i] = lighting_power;

//     // START TESTING.

//     let hs_front = simple_model.surfaces[0]
//         .front_convection_coefficient(&state)
//         .unwrap();
//     let hs_back = simple_model.surfaces[0]
//         .back_convection_coefficient(&state)
//         .unwrap();
//     let hs_front = 10.;
//     let hs_back = 10.;
//     let r = thermal_model.surfaces[0].discretization.r_value() + 1. / hs_front + 1. / hs_back;

//     // Initial T of the zone
//     let t_start = 22.;

//     thermal_model.zones[0]
//         .reference_space
//         .set_dry_bulb_temperature(&mut state, t_start);

//     let t_out: Float = 30.0; // T of surroundings

//     // test model
//     let tester = SingleZoneTestModel {
//         zone_volume,
//         surface_area, // the window is a hole on the wall... does not add area
//         lighting_power,
//         facade_r: r,
//         temp_out: t_out,
//         temp_start: t_start,
//         ..SingleZoneTestModel::default()
//     };
//     let exp_fn = tester.get_closed_solution();

//     let mut weather = SyntheticWeather::default();
//     weather.dry_bulb_temperature = Box::new(ScheduleConstant::new(t_out));
//     weather.wind_direction = Box::new(ScheduleConstant::new(0.0));
//     weather.wind_speed = Box::new(ScheduleConstant::new(0.0));

//     let dt = main_dt; // / model.dt_subdivisions() as Float;

//     let mut date = Date {
//         day: 1,
//         hour: 0.0,
//         month: 1,
//     };

//     // March:
//     let n = 800;
//     let mut exp = Vec::with_capacity(n);
//     let mut found = Vec::with_capacity(n);
//     for i in 0..n {
//         let time = (i as Float) * dt;
//         date.add_seconds(time);

//         let found_v = thermal_model.zones[0]
//             .reference_space
//             .dry_bulb_temperature(&state)
//             .unwrap();

//         thermal_model
//             .march(date, &weather, &simple_model, &mut state)
//             .unwrap();

//         // Get exact solution.
//         let exp_v = exp_fn(time);

//         exp.push(exp_v);
//         found.push(found_v);
//     }

//     (exp, found)
// }

// fn march_with_window_and_heater() -> (Vec<Float>, Vec<Float>) {
//     let surface_area = 4.;
//     let zone_volume = 40.;
//     let heating_power = 100.;

//     let (simple_model, mut state_header) = get_single_zone_test_building(
//         // &mut state,
//         &SingleZoneTestBuildingOptions {
//             zone_volume,
//             surface_area,
//             heating_power,
//             construction: vec![TestMat::Polyurethane(0.02)],
//             emmisivity: 0.0,
//             ..Default::default()
//         },
//     );

//     // Finished model the SimpleModel

//     let n: usize = 20;
//     let main_dt = 60. * 60. / n as Float;
//     let thermal_model = ThermalModel::new(&META_OPTIONS, (),&simple_model, &mut state_header, n).unwrap();

//     let mut state = state_header.take_values().unwrap();
//     // MAP THE STATE
//     // model.map_simulation_state(&mut state).unwrap();

//     // turn the heater on
//     if let HVAC::ElectricHeater(heater) = &simple_model.hvacs[0] {
//         let hvac_state_i = heater.heating_cooling_consumption_index().unwrap();
//         state[hvac_state_i] = heating_power;
//     }

//     // START TESTING.
//     // assert!(!model.surfaces[0].is_massive());

//     let hs_front = simple_model.surfaces[0]
//         .front_convection_coefficient(&state)
//         .unwrap();
//     let hs_back = simple_model.surfaces[0]
//         .back_convection_coefficient(&state)
//         .unwrap();
//     let r = thermal_model.surfaces[0].discretization.r_value() + 1. / hs_front + 1. / hs_back;

//     // Initial T of the zone
//     let t_start = thermal_model.zones[0]
//         .reference_space
//         .dry_bulb_temperature(&state)
//         .unwrap();
//     let t_out: Float = 30.0; // T of surroundings

//     // test model
//     let tester = SingleZoneTestModel {
//         zone_volume,
//         surface_area, // the window is a hole on the wall... does not add area
//         heating_power,
//         facade_r: r,
//         temp_out: t_out,
//         temp_start: t_start,
//         ..SingleZoneTestModel::default()
//     };
//     let exp_fn = tester.get_closed_solution();

//     let mut weather = SyntheticWeather::default();
//     weather.dry_bulb_temperature = Box::new(ScheduleConstant::new(t_out));
//     weather.wind_direction = Box::new(ScheduleConstant::new(0.0));
//     weather.wind_speed = Box::new(ScheduleConstant::new(0.0));

//     let dt = main_dt; // / model.dt_subdivisions() as Float;

//     let mut date = Date {
//         day: 1,
//         hour: 0.0,
//         month: 1,
//     };

//     // March:
//     let n = 800;
//     let mut exp = Vec::with_capacity(n);
//     let mut found = Vec::with_capacity(n);
//     for i in 0..n {
//         let time = (i as Float) * dt;
//         date.add_seconds(time);

//         let found_v = thermal_model.zones[0]
//             .reference_space
//             .dry_bulb_temperature(&state)
//             .unwrap();

//         thermal_model
//             .march(date, &weather, &simple_model, &mut state)
//             .unwrap();

//         // Get exact solution.
//         let exp_v = exp_fn(time);

//         exp.push(exp_v);
//         found.push(found_v);
//     }
//     (exp, found)
// }

// fn march_with_window_heater_and_infiltration() -> (Vec<Float>, Vec<Float>) {
//     let surface_area = 4.;
//     let zone_volume = 40.;
//     let heating_power = 10.;
//     let infiltration_rate = 0.1;
//     let t_out: Float = 30.0; // T of surroundings

//     let (simple_model, mut state_header) = get_single_zone_test_building(
//         // &mut state,
//         &SingleZoneTestBuildingOptions {
//             zone_volume,
//             surface_area,
//             heating_power,
//             infiltration_rate,
//             emmisivity: 0.0,
//             construction: vec![TestMat::Polyurethane(0.02)],
//             ..Default::default()
//         },
//     );

//     // Finished model the SimpleModel

//     let n: usize = 20;
//     let main_dt = 60. * 60. / n as Float;
//     let thermal_model = ThermalModel::new(&META_OPTIONS, (),&simple_model, &mut state_header, n).unwrap();

//     // Set infiltration
//     let inf_vol_index = state_header.push(
//         SimulationStateElement::SpaceInfiltrationVolume(0),
//         infiltration_rate,
//     );
//     simple_model.spaces[0].set_infiltration_volume_index(inf_vol_index);
//     let inf_temp_index = state_header.push(
//         SimulationStateElement::SpaceInfiltrationTemperature(0),
//         t_out,
//     );
//     simple_model.spaces[0].set_infiltration_temperature_index(inf_temp_index);

//     // MAP THE STATE

//     let mut state = state_header.take_values().unwrap();

//     // turn the heater on
//     if let HVAC::ElectricHeater(heater) = &simple_model.hvacs[0] {
//         let hvac_state_i = heater.heating_cooling_consumption_index().unwrap();
//         state[hvac_state_i] = heating_power;
//     }

//     // START TESTING.

//     let hs_front = simple_model.surfaces[0]
//         .front_convection_coefficient(&state)
//         .unwrap();
//     let hs_back = simple_model.surfaces[0]
//         .back_convection_coefficient(&state)
//         .unwrap();
//     let r = thermal_model.surfaces[0].discretization.r_value() + 1. / hs_front + 1. / hs_back;

//     // Initial T of the zone
//     let t_start = thermal_model.zones[0]
//         .reference_space
//         .dry_bulb_temperature(&state)
//         .unwrap();

//     // test model
//     let tester = SingleZoneTestModel {
//         zone_volume,
//         surface_area, // the window is a hole on the wall... does not add area
//         heating_power,
//         facade_r: r,
//         temp_out: t_out,
//         temp_start: t_start,
//         infiltration_rate,
//         ..SingleZoneTestModel::default()
//     };
//     let exp_fn = tester.get_closed_solution();

//     let mut weather = SyntheticWeather::default();
//     weather.dry_bulb_temperature = Box::new(ScheduleConstant::new(t_out));
//     weather.wind_direction = Box::new(ScheduleConstant::new(0.0));
//     weather.wind_speed = Box::new(ScheduleConstant::new(0.0));

//     let dt = main_dt; // / model.dt_subdivisions() as Float;

//     let mut date = Date {
//         day: 1,
//         hour: 0.0,
//         month: 1,
//     };

//     // March:
//     let n = 22;
//     let mut exp = Vec::with_capacity(n);
//     let mut found = Vec::with_capacity(n);
//     for i in 0..n {
//         let time = (i as Float) * dt;
//         date.add_seconds(time);

//         let found_v = thermal_model.zones[0]
//             .reference_space
//             .dry_bulb_temperature(&state)
//             .unwrap();

//         thermal_model
//             .march(date, &weather, &simple_model, &mut state)
//             .unwrap();

//         // Get exact solution.
//         let exp_v = exp_fn(time);

//         exp.push(exp_v);
//         found.push(found_v);
//     }
//     (exp, found)
// }

fn march_one_wall(
    dir: &'static str,
    emmisivity: Float,
    solar_abs: Float,
    construction: Vec<TestMat>,
) -> (Vec<Float>, Vec<Float>) {
    let surface_area = 20. * 3.;
    let zone_volume = 600.;

    let (simple_model, mut state_header) = get_single_zone_test_building(
        // &mut state,
        &SingleZoneTestBuildingOptions {
            zone_volume,
            surface_area,
            construction,
            emmisivity,
            solar_absorbtance: solar_abs,
            ..Default::default()
        },
    );

    // Finished model the SimpleModel

    let n: usize = 20;
    // let main_dt = 60. * 60. / n as Float;
    let thermal_model =
        ThermalModel::new(&META_OPTIONS, (), &simple_model, &mut state_header, n).unwrap();

    let mut state = state_header.take_values().unwrap();

    let path_string = format!("./tests/{}/eplusout.csv", dir);
    let path = path_string.as_str();
    let cols = validate::from_csv(path, &[1, 2, 3, 7, 8, 10, 11]);

    let wind_speed = &cols[0]; // 1
    let wind_direction = &cols[1]; // 2
    let incident_solar_radiation = &cols[2]; // 3
    let indoor_thermal_heat_gain = &cols[3]; // 5
    let outdoor_temp = &cols[4]; // 6
    let outdoor_thermal_heat_gain = &cols[5]; // 8
    let exp_zone_air_temp = &cols[6]; // 9

    // Set initial temperature
    simple_model.spaces[0].set_dry_bulb_temperature(&mut state, exp_zone_air_temp[0]);

    let mut date = Date {
        month: 1,
        day: 1,
        hour: 0.0,
    };
    let n = outdoor_temp.len();
    let mut exp = Vec::with_capacity(n);
    let mut found = Vec::with_capacity(n);
    for i in 0..n {
        // Get zone's temp
        let found_temp = simple_model.spaces[0].dry_bulb_temperature(&state).unwrap();
        let exp_temp = exp_zone_air_temp[i];
        if i > 5000 {
            // skip warmup
            exp.push(exp_temp);
            found.push(found_temp);
        }

        // Set outdoor temp
        let mut weather = SyntheticWeather::default();
        weather.dry_bulb_temperature = Box::new(ScheduleConstant::new(outdoor_temp[i]));
        weather.wind_direction = Box::new(ScheduleConstant::new(wind_direction[i]));
        weather.wind_speed = Box::new(ScheduleConstant::new(wind_speed[i]));

        let surface = &simple_model.surfaces[0];

        // Set Solar Radiation
        surface.set_back_incident_solar_irradiance(&mut state, incident_solar_radiation[i]);

        // Set Long Wave radiation
        if emmisivity > 1e-3 {
            let ts = surface.last_node_temperature(&state).unwrap();
            let v = outdoor_thermal_heat_gain[i] / surface_area / emmisivity
                + heat::SIGMA * (ts + 273.15).powi(4);
            surface.set_back_ir_irradiance(&mut state, v);

            let ts = surface.first_node_temperature(&state).unwrap();
            let v = indoor_thermal_heat_gain[i] / surface_area / emmisivity
                + heat::SIGMA * (ts + 273.15).powi(4);
            surface.set_front_ir_irradiance(&mut state, v);
        }

        // March
        thermal_model
            .march(date, &weather, &simple_model, &mut state)
            .unwrap();

        // Advance
        date.add_hours(1. / n as Float);
    }
    (exp, found)
}

// fn theoretical(validations: &mut Validator) {
//     const EXPECTED_LEGEND : &'static str = "Theoretical Solution";

//     #[valid(Nomass Wall - Walls only)]
//     fn nomass_wallonly()-> Box<dyn Validate> {
//         let (expected, found) = very_simple_march();
//         get_validator(expected, found, EXPECTED_LEGEND)
//     }

//     #[valid(Nomass Wall - Walls and Fenestration)]
//     fn nomass_wall_and_window()-> Box<dyn Validate> {
//         let (expected, found) = march_with_window();
//         get_validator(expected, found, EXPECTED_LEGEND)
//     }

//     #[valid(Nomass Wall - Walls and Fenestration, with Luminaire on)]
//     fn window_and_luminaire()-> Box<dyn Validate> {
//         let (expected, found) = march_with_window_and_luminaire();
//         get_validator(expected, found, EXPECTED_LEGEND)
//     }

//     #[valid(Nomass Wall - Walls and Window and heater)]
//     fn nomass_wall_and_window_and_heater()-> Box<dyn Validate> {
//         let (expected, found) = march_with_window_and_heater();
//         get_validator(expected, found, EXPECTED_LEGEND)
//     }

//     #[valid(Nomass Wall - Walls and Fenestration, with heater on and infiltration)]
//     fn window_heater_and_infiltration()-> Box<dyn Validate> {
//         let (expected, found) = march_with_window_heater_and_infiltration();
//         get_validator(expected, found, EXPECTED_LEGEND)
//     }

//     // validations.push(nomass_wallonly());
//     // validations.push(nomass_wall_and_window());
//     validations.push(window_and_luminaire());
//     // validations.push(nomass_wall_and_window_and_heater());
//     // validations.push(window_heater_and_infiltration());

// }

fn massive(validations: &mut Validator) {
    const EXPECTED_LEGEND: &'static str = "EnergyPlus";

    #[valid(Massive Wall, with Solar and Long Wave Radiation)]
    fn wall1() -> Box<dyn Validate> {
        let (expected, found) =
            march_one_wall("massive_full", 0.9, 0.7, vec![TestMat::Concrete(0.2)]);
        get_validator(expected, found, EXPECTED_LEGEND)
    }

    #[valid(Massive Wall, with no Solar or Long Wave Radiation)]
    fn wall2() -> Box<dyn Validate> {
        // Massive, NO solar and NO Long Wave
        let (expected, found) = march_one_wall(
            "massive_no_ir_no_solar",
            0.0,
            0.0,
            vec![TestMat::Concrete(0.2)],
        );
        get_validator(expected, found, EXPECTED_LEGEND)
    }

    #[valid(Massive Wall, with Solar Radiation but not Long Wave Radiation)]
    fn wall3() -> Box<dyn Validate> {
        // Massive, WITH  solar and NO Long Wave
        let (expected, found) = march_one_wall(
            "massive_no_ir_yes_solar",
            0.0,
            0.7,
            vec![TestMat::Concrete(0.2)],
        );
        get_validator(expected, found, EXPECTED_LEGEND)
    }

    #[valid(Massive Wall, with Long Wave Radiation but not Solar Radiation)]
    fn wall4() -> Box<dyn Validate> {
        // Massive, No  solar and WITH Long Wave
        let (expected, found) = march_one_wall(
            "massive_yes_ir_no_solar",
            0.9,
            0.0,
            vec![TestMat::Concrete(0.2)],
        );
        get_validator(expected, found, EXPECTED_LEGEND)
    }

    validations.push(wall1());
    validations.push(wall2());
    validations.push(wall3());
    validations.push(wall4());
}

fn mixed(validations: &mut Validator) {
    const EXPECTED_LEGEND: &'static str = "EnergyPlus";

    #[valid(Mixed Mass Wall, with Solar Radiation and Long Wave Radiation)]
    fn wall1() -> Box<dyn Validate> {
        // Mixed Mass, With solar Radiation and Long Wave
        let (expected, found) = march_one_wall(
            "mixed_full",
            0.9,
            0.7,
            vec![
                TestMat::Polyurethane(0.02),
                TestMat::Concrete(0.2),
                TestMat::Polyurethane(0.02),
            ],
        );
        get_validator(expected, found, EXPECTED_LEGEND)
    }

    #[valid(Mixed Mass Wall, without Solar or Long Wave Radiation)]
    fn wall2() -> Box<dyn Validate> {
        // Mixed Mass, NO solar and NO Long Wave
        let (expected, found) = march_one_wall(
            "mixed_no_ir_no_solar",
            0.0,
            0.0,
            vec![
                TestMat::Polyurethane(0.02),
                TestMat::Concrete(0.2),
                TestMat::Polyurethane(0.02),
            ],
        );
        get_validator(expected, found, EXPECTED_LEGEND)
    }

    #[valid(Mixed Mass Wall, with Solar Radiation but no Long Wave Radiation)]
    fn wall3() -> Box<dyn Validate> {
        // Mixed Mass, WITH  solar and NO Long Wave
        let (expected, found) = march_one_wall(
            "mixed_no_ir_yes_solar",
            0.0,
            0.7,
            vec![
                TestMat::Polyurethane(0.02),
                TestMat::Concrete(0.2),
                TestMat::Polyurethane(0.02),
            ],
        );
        get_validator(expected, found, EXPECTED_LEGEND)
    }

    #[valid(Mixed Mass Wall, with Long Wave Radiation but no Solar Radiation)]
    fn wall4() -> Box<dyn Validate> {
        // Mixed Mass, No  solar and WITH Long Wave
        let (expected, found) = march_one_wall(
            "mixed_yes_ir_no_solar",
            0.9,
            0.0,
            vec![
                TestMat::Polyurethane(0.02),
                TestMat::Concrete(0.2),
                TestMat::Polyurethane(0.02),
            ],
        );
        get_validator(expected, found, EXPECTED_LEGEND)
    }

    validations.push(wall1());
    // validations.push(wall2());
    // validations.push(wall3());
    // validations.push(wall4());
}

fn nomass(validations: &mut Validator) {
    const EXPECTED_LEGEND: &'static str = "EnergyPlus";

    #[valid(No Mass Wall, with Solar Radiation and Long Wave Radiation)]
    fn wall1() -> Box<dyn Validate> {
        // No Mass, With solar Radiation and Long Wave
        let (expected, found) =
            march_one_wall("nomass_full", 0.9, 0.7, vec![TestMat::Polyurethane(0.02)]);
        get_validator(expected, found, EXPECTED_LEGEND)
    }

    #[valid(No Mass Wall, without Solar or Long Wave Radiation)]
    fn wall2() -> Box<dyn Validate> {
        // No Mass, NO solar and NO Long Wave
        let (expected, found) = march_one_wall(
            "nomass_no_ir_no_solar",
            0.0,
            0.0,
            vec![TestMat::Polyurethane(0.02)],
        );
        get_validator(expected, found, EXPECTED_LEGEND)
    }

    #[valid(No Mass Wall, with Solar Radiation but no Long Wave Radiation)]
    fn wall3() -> Box<dyn Validate> {
        // No Mass, WITH  solar and NO Long Wave
        let (expected, found) = march_one_wall(
            "nomass_no_ir_yes_solar",
            0.0,
            0.7,
            vec![TestMat::Polyurethane(0.02)],
        );
        get_validator(expected, found, EXPECTED_LEGEND)
    }

    #[valid(No Mass Wall, with Long Wave Radiation but no Solar Radiation)]
    fn wall4() -> Box<dyn Validate> {
        // No Mass, No  solar and WITH Long Wave
        let (expected, found) = march_one_wall(
            "nomass_yes_ir_no_solar",
            0.9,
            0.0,
            vec![TestMat::Polyurethane(0.02)],
        );

        get_validator(expected, found, EXPECTED_LEGEND)
    }

    validations.push(wall1());
    validations.push(wall2());
    validations.push(wall3());
    validations.push(wall4());
}

// fn march_trombe_wall(
//     dir: &'static str,
//     emmisivity: Float,
//     solar_abs: Float,
//     construction: Vec<TestMat>,
// ) -> (Vec<Float>, Vec<Float>) {
//     let surface_area = 20. * 3.;
//     let zone_volume = 600.;

//     let (simple_model, mut state_header) = get_single_zone_test_building(
//         // &mut state,
//         &SingleZoneTestBuildingOptions {
//             zone_volume,
//             surface_area,
//             construction,
//             emmisivity,
//             solar_absorbtance: solar_abs,
//             ..Default::default()
//         },
//     );

//     // Finished model the SimpleModel

//     let n: usize = 20;
//     // let main_dt = 60. * 60. / n as Float;
//     let thermal_model = ThermalModel::new(&simple_model, &mut state_header, n).unwrap();

//     let mut state = state_header.take_values().unwrap();

//     let path_string = format!("./tests/{}/eplusout.csv", dir);
//     let path = path_string.as_str();
//     let cols = validate::from_csv(path, &[3, 15, 17, 18, 24]);
//     let incident_solar_radiation = &cols[0]; //3
//     let indoor_thermal_heat_gain = &cols[1]; //15
//     let outdoor_temp = &cols[2]; //17
//     let outdoor_thermal_heat_gain = &cols[3]; //18
//     let exp_zone_air_temp = &cols[4]; //24

//     // Set initial temperature
//     simple_model.spaces[0].set_dry_bulb_temperature(&mut state, exp_zone_air_temp[0]);

//     let mut date = Date {
//         month: 1,
//         day: 1,
//         hour: 0.0,
//     };
//     let n = outdoor_temp.len();
//     let mut exp = Vec::with_capacity(n);
//     let mut found = Vec::with_capacity(n);
//     for i in 0..n {
//         // Get zone's temp
//         let found_temp = simple_model.spaces[0].dry_bulb_temperature(&state).unwrap();
//         let exp_temp = exp_zone_air_temp[i];
//         if i > 000 {
//             // skip warmup
//             exp.push(exp_temp);
//             found.push(found_temp);
//         }

//         // Set outdoor temp
//         let mut weather = SyntheticWeather::default();
//         weather.dry_bulb_temperature = Box::new(ScheduleConstant::new(outdoor_temp[i]));
//            weather.wind_direction = Box::new(ScheduleConstant::new(0.0));

//         let surface = &simple_model.surfaces[0];

//         // Set Solar Radiation
//         surface.set_back_incident_solar_irradiance(&mut state, incident_solar_radiation[i]);

//         // Set Long Wave radiation
//         if emmisivity > 1e-3 {
//             let ts = surface.last_node_temperature(&state).unwrap();
//             let v = outdoor_thermal_heat_gain[i] / surface_area / emmisivity
//                 + heat::SIGMA * (ts + 273.15).powi(4);
//             surface.set_back_ir_irradiance(&mut state, v);

//             let ts = surface.first_node_temperature(&state).unwrap();
//             let v = indoor_thermal_heat_gain[i] / surface_area / emmisivity
//                 + heat::SIGMA * (ts + 273.15).powi(4);
//             surface.set_front_ir_irradiance(&mut state, v);
//         }

//         // March
//         thermal_model
//             .march(date, &weather, &simple_model, &mut state)
//             .unwrap();

//         // Advance
//         date.add_hours(1. / n as Float);
//     }
//     (exp, found)
// }

// fn trombe_wall(validations: &mut Validator) {
//     // No Mass, With solar Radiation and Long Wave
//     let (expected, found) = march_trombe_wall(
//         "trombe_wall_full",
//         0.9,
//         0.08,
//         vec![
//             TestMat::Concrete(0.2),
//             TestMat::Air(0.05),
//             TestMat::Glass(0.03, 0.82),
//         ],
//     );
//     let v = validate::SeriesValidator {
//         title: "Trombe Wall, with Solar Radiation and Long Wave Radiation",
//         x_label: Some("time step"),
//         y_label: Some("Zone Temperature"),
//         y_units: Some("C"),
//         found_name: "Simple",
//         expected_name: "EnergyPlus",

//         expected,
//         found,

//         ..validate::SeriesValidator::default()
//     };
//     validations.push(Box::new(v));
// }

#[test]
fn validate() {
    // cargo test --package heat --test validate -- validate --exact --nocapture
    let p = "./docs/validation";
    if !std::path::Path::new(&p).exists() {
        std::fs::create_dir(p).unwrap();
    }

    let target_file = format!("{}/walls.html", p);
    let mut validations = Validator::new("SIMPLE Thermal validation report", &target_file);

    // theoretical(&mut validations);
    massive(&mut validations);
    mixed(&mut validations);
    nomass(&mut validations);
    // trombe_wall(&mut validations);
    validations.validate().unwrap();
}
//

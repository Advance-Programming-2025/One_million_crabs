use common_game::components::planet::{
    DummyPlanetState, Planet, PlanetAI, PlanetState, PlanetType,
};
use common_game::components::resource::BasicResourceType::*;
use common_game::components::resource::{
    BasicResource, BasicResourceType, Combinator, ComplexResource, ComplexResourceRequest,
    ComplexResourceType, Generator, GenericResource,
};
use common_game::components::rocket::Rocket;
use common_game::components::sunray::Sunray;
use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
use common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};
use crossbeam_channel::{Receiver, Sender};


use common_game::logging::{ActorType, Participant};
use common_game::utils::ID;


use logging_utils::{get_receiver_id, get_sender_id, log_explorer_to_planet, log_fn_call, log_internal_op, log_orch_to_planet, LoggableActor};
///////////////////////////////////////////////////////////////////////////////////////////
// CrabRave Constructor
///////////////////////////////////////////////////////////////////////////////////////////


//This function will be called by the Orchestrator
pub fn create_planet(
    rx_orchestrator: Receiver<OrchestratorToPlanet>,
    tx_orchestrator: Sender<PlanetToOrchestrator>,
    rx_explorer: Receiver<ExplorerToPlanet>,
    planet_id: u32,
) -> Result<Planet, String> {
    //LOG
    log_fn_call!(dir
        ActorType::Planet,
        planet_id,
        "create_planet()";
        "rx_orchestrator"=>format!("memory address: {:?}",get_receiver_id(&rx_orchestrator)),
        "tx_orchestrator"=>format!("memory address: {:?}",get_sender_id(&tx_orchestrator)),
        "rx_explorer"=>format!("memory address: {:?}",get_receiver_id(&rx_explorer)),
    );
    //LOG
    let (planet_type, ai, gen_rules, comb_rules, orchestrator_channels, explorer_channels) = (
        PlanetType::D,
        OneMillionCrabs::new(planet_id),
        vec![Carbon, Hydrogen, Oxygen, Silicon],
        vec![],
        (rx_orchestrator, tx_orchestrator),
        rx_explorer,
    );

    //LOG
    let gen_rules_str: String = gen_rules.iter().map(|x| x.to_string_2() + ", ").collect();
    let comb_rules_str: String = gen_rules.iter().map(|x| x.to_string_2() + ", ").collect();
    //LOG

    let new_planet = Planet::new(
        planet_id,
        planet_type,
        Box::new(ai),
        gen_rules,
        comb_rules,
        orchestrator_channels,
        explorer_channels,
    )?;

    //LOG
    log_internal_op!(dir
        ActorType::Planet,
        planet_id,
        "action"=>"new planet created",
        "planet_id"=>planet_id,
        "planet_type"=>format!("{:?}",planet_type),
        "gen_rules"=>format!("{:?}",gen_rules_str),
        "comb_rules"=>format!("{:?}",comb_rules_str),
    );
    //LOG

    Ok(new_planet)
}

///////////////////////////////////////////////////////////////////////////////////////////
// PlanetAI
///////////////////////////////////////////////////////////////////////////////////////////

pub struct OneMillionCrabs{
    my_id: u32,
}

impl LoggableActor for OneMillionCrabs{
    fn actor_type(&self) -> ActorType {
        ActorType::Planet
    }

    fn actor_id(&self) -> u32 {
        self.my_id
    }
}

impl OneMillionCrabs {
    fn new(planet_id: u32) -> Self {
        //LOG
        log_fn_call!( dir
            ActorType::Planet,
            planet_id,
            "OneMillionCrabs::new()",
            planet_id,
        );
        //LOG
        //initialize_free_cell_stack(planet_id);
        Self{my_id:planet_id}
    }
}

impl PlanetAI for OneMillionCrabs {
    fn handle_sunray(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
        sunray: Sunray,
    ) {
        //LOG
        let mut result_str=String::new();
        //LOG
        if state.charge_cell(sunray).is_none() {
            //LOG
            log_internal_op!(
                self,
                "action"=>"new cell charged",
            );
            result_str= "New cell charged".to_string();
            //LOG
        } else {
            result_str.push_str("No free cell found");
        }

        //LOG
        log_orch_to_planet!(
            self,
            "handle_sunray()";
            "state"=>format!("{:?}",PlanetState::to_dummy(state)),
            "_generator"=>"&Generator",
            "_combinator"=>"&Combinator",
            "sunray"=>"Sunray";
            result = result_str
        );
        //LOG
    }

    /// Handler used to determine the strategy in case of an incoming asteroid.
    /// It will usually try to build a rocket if it can and if it has any
    /// energy cells available.
    /// As for our planet, it's a type D, so the planet will ALWAYS die
    /// when it gets an asteroid. Any other behavior is unexpected and
    /// should be reported.
    /// Refer to the common crate documentation for more info on the
    /// default behavior of this function.
    fn handle_asteroid(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
    ) -> Option<Rocket> {
        //if the planet can't build rockets, you're screwed
        let mut result_str=String::new();

        let mut ris = None;
        if !state.can_have_rocket() {
            ris = None;
            //LOG
            result_str=String::from("The planet cannot have a rocket");
            //LOG
        }
        //LOG

        log_orch_to_planet!(
            self,
            "handle_asteroid()";
            "state"=>format!("{:?}",PlanetState::to_dummy(state)),
            "_generator"=>"&Generator",
            "_combinator"=>"&Combinator";
            result = result_str
        );

        //LOG

        ris
        //shouldn't be able to get here, but just in case...
        //None
    }

    fn handle_internal_state_req(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
    ) -> DummyPlanetState {
        //LOG
        log_orch_to_planet!(
            self,
            "handle_internal_state_req()";
            "state"=>format!("{:?}",PlanetState::to_dummy(state)),
            "_generator"=>"&Generator",
            "_combinator"=>"&Combinator";
            result = format!("{:?}",PlanetState::to_dummy(state))
        );
        //LOG
        state.to_dummy()
    }

    fn handle_explorer_msg(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
        msg: ExplorerToPlanet,
    ) -> Option<PlanetToExplorer> {
        //LOG
        log_internal_op!(
            self,
            "fn"=>"handle_explorer_msg()",
            "msg"=>format!("{:?}",msg),
        );
        //LOG

        match msg {
            ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id: id } => {
                // restituisce la prima cell carica, se c'è

                let mut n_available_cells = 0;
                for i in 0..N_CELLS {
                    if state.cell(i).is_charged() {
                        n_available_cells += 1;
                    }
                }

                let ris = Some(PlanetToExplorer::AvailableEnergyCellResponse {
                    available_cells: n_available_cells,
                });

                //LOG
                log_explorer_to_planet!(
                    self,
                    id,
                    "handle_explorer_msg()";
                    "state"=>format!("{:?}",PlanetState::to_dummy(state)),
                    "_generator"=>"&Generator",
                    "_combinator"=>"&Combinator",
                    "msg" => format!("{:?}", msg);
                    result = format!("{} energy cell available", n_available_cells)
                );
                //LOG
                ris
            }
            ExplorerToPlanet::SupportedResourceRequest { explorer_id: id } => {
                //LOG
                log_internal_op!(
                    self,
                    "action"=>"generator.all_available_recipes()"
                );

                log_explorer_to_planet!(
                    self,
                    id,
                    "handle_explorer_msg()";
                    "state"=>format!("{:?}",PlanetState::to_dummy(state)),
                    "_generator"=>"&Generator",
                    "_combinator"=>"&Combinator",
                    "msg" => format!("{:?}", msg);
                    result = format!("Supported resouces: {:?}", generator.all_available_recipes())
                );
                //LOG

                Some(PlanetToExplorer::SupportedResourceResponse {
                    resource_list: generator.all_available_recipes(),
                })
            }
            ExplorerToPlanet::SupportedCombinationRequest { explorer_id: id } => {
                //LOG
                log_internal_op!(
                    self,
                    "action"=>"combinator.all_available_recipes()"
                );

                log_explorer_to_planet!(
                    self,
                    id,
                    "handle_explorer_msg()";
                    "state"=>format!("{:?}",PlanetState::to_dummy(state)),
                    "_generator"=>"&Generator",
                    "_combinator"=>"&Combinator",
                    "msg" => format!("{:?}", msg);
                    result = format!("Supported resouces: {:?}", combinator.all_available_recipes())
                );
                //LOG
                Some(PlanetToExplorer::SupportedCombinationResponse {
                    combination_list: combinator.all_available_recipes(),
                })
            }

            //TODO use explorer_id to send the gen resource to correct Explorer
            ExplorerToPlanet::GenerateResourceRequest {
                explorer_id,
                resource,
            } => {
                //LOG
                let result_str;
                //LOG
                let mut res = Some(PlanetToExplorer::GenerateResourceResponse { resource: None });
                let requested_resource = resource;
                // controllo se c'è una cella carica

                if let Some((cell, idx)) = state.full_cell() {
                    //LOG
                    //LOG
                    // pattern matching per generare la risorsa corretta
                    let generated_resource = match requested_resource {
                        BasicResourceType::Carbon => {
                            generator.make_carbon(cell).map(BasicResource::Carbon)
                        } // make_ controlla già se la risorsa è presente in generator
                        BasicResourceType::Silicon => {
                            generator.make_silicon(cell).map(BasicResource::Silicon)
                        }
                        BasicResourceType::Oxygen => {
                            generator.make_oxygen(cell).map(BasicResource::Oxygen)
                        }
                        BasicResourceType::Hydrogen => {
                            generator.make_hydrogen(cell).map(BasicResource::Hydrogen)
                        }
                    };

                    //LOG
                    //LOG

                    // verifico il risultato di state.generator.make...
                    match generated_resource {
                        Ok(resource) => {
                            //LOG
                            result_str=format!("Resource created: {:?}, using energy cell at index: {}",resource, idx);
                            //LOG
                            res = Some(PlanetToExplorer::GenerateResourceResponse {
                                resource: Some(resource),
                            });
                        }
                        Err(err) => {
                            //LOG
                            result_str=format!("cannot create resource {:?}. Error: {}", resource, err);
                            //LOG
                        }
                    }
                }
                else{
                    result_str=String::from("No energy cell available");
                }
                //LOG
                log_explorer_to_planet!(
                    self,
                    explorer_id,
                    "handle_explorer_msg()";
                    "state"=>format!("{:?}",PlanetState::to_dummy(state)),
                    "_generator"=>"&Generator",
                    "_combinator"=>"&Combinator",
                    "msg" => format!("{:?}", msg);
                    result = result_str
                );
                //LOG
                res
            }
            //TODO use explorer_id to send the gen resource to correct Explorer
            ExplorerToPlanet::CombineResourceRequest {
                explorer_id,
                msg: resource,
                //renamed msg to resouce to be more consistent with generateresourcerequest
            } => {
                //LOG
                let result_str;
                let appo=format!("{:?}", resource);
                //LOG

                let res;

                // searching the index of the first free cell
                if let Some((cell, cell_idx)) = state.full_cell() {
                    //LOG
                    //LOG
                    let complex_resource: Result<
                        ComplexResource,
                        (String, GenericResource, GenericResource),
                    > = match resource {
                        ComplexResourceRequest::Water(r1, r2) => {
                            log_internal_op!(self, "action"=>"combinator.make_water");
                            combinator
                                .make_water(r1, r2, cell)
                                .map(ComplexResource::Water)
                                .map_err(|(e, r1, r2)| {
                                    (
                                        e,
                                        GenericResource::BasicResources(BasicResource::Hydrogen(
                                            r1,
                                        )),
                                        GenericResource::BasicResources(BasicResource::Oxygen(r2)),
                                    )
                                })
                        }
                        ComplexResourceRequest::Diamond(r1, r2) => {
                            log_internal_op!(self, "action"=>"combinator.make_diamond");
                            combinator
                                .make_diamond(r1, r2, cell)
                                .map(ComplexResource::Diamond)
                                .map_err(|(e, r1, r2)| {
                                    (
                                        e,
                                        GenericResource::BasicResources(BasicResource::Carbon(r1)),
                                        GenericResource::BasicResources(BasicResource::Carbon(r2)),
                                    )
                                })
                        }
                        ComplexResourceRequest::Life(r1, r2) => {
                            log_internal_op!(self, "action"=>"combinator.make_life");
                            combinator
                                .make_life(r1, r2, cell)
                                .map(ComplexResource::Life)
                                .map_err(|(e, r1, r2)| {
                                    (
                                        e,
                                        GenericResource::ComplexResources(ComplexResource::Water(
                                            r1,
                                        )),
                                        GenericResource::BasicResources(BasicResource::Carbon(r2)),
                                    )
                                })
                        }

                        ComplexResourceRequest::Robot(r1, r2) => {
                            log_internal_op!(self, "action"=>"combinator.make_robot");
                            combinator
                                .make_robot(r1, r2, cell)
                                .map(ComplexResource::Robot)
                                .map_err(|(e, r1, r2)| {
                                    (
                                        e,
                                        GenericResource::BasicResources(BasicResource::Silicon(r1)),
                                        GenericResource::ComplexResources(ComplexResource::Life(
                                            r2,
                                        )),
                                    )
                                })
                        }

                        ComplexResourceRequest::Dolphin(r1, r2) => {
                            log_internal_op!(self, "action"=>"combinator.make_dolphin");
                            combinator
                                .make_dolphin(r1, r2, cell)
                                .map(ComplexResource::Dolphin)
                                .map_err(|(e, r1, r2)| {
                                    (
                                        e,
                                        GenericResource::ComplexResources(ComplexResource::Water(
                                            r1,
                                        )),
                                        GenericResource::ComplexResources(ComplexResource::Life(
                                            r2,
                                        )),
                                    )
                                })
                        }

                        ComplexResourceRequest::AIPartner(r1, r2) => {
                            log_internal_op!(self, "action"=>"combinator.make_aipartner");
                            combinator
                                .make_aipartner(r1, r2, cell)
                                .map(ComplexResource::AIPartner)
                                .map_err(|(e, r1, r2)| {
                                    (
                                        e,
                                        GenericResource::ComplexResources(ComplexResource::Robot(
                                            r1,
                                        )),
                                        GenericResource::ComplexResources(
                                            ComplexResource::Diamond(r2),
                                        ),
                                    )
                                })
                        }
                    };

                    // checking the result of complex_resource
                    match complex_resource {
                        Ok(resource) => {
                            //LOG
                            result_str=String::from("Complex resource created");
                            //LOG

                            res = Some(PlanetToExplorer::CombineResourceResponse {
                                complex_response: Ok(resource),
                            });
                        }
                        Err(err) => {
                            //LOG
                            result_str=format!("Complex resource not created. Error: {:?}", err.2);
                            //LOG

                            res = Some(PlanetToExplorer::CombineResourceResponse {
                                complex_response: Err(err),
                            });
                        }
                    }
                } else {
                    //LOG
                    result_str=String::from("No energy cell available");
                    //LOG

                    let (ret1, ret2) = match resource {
                        ComplexResourceRequest::Water(r1, r2) => (
                            GenericResource::BasicResources(BasicResource::Hydrogen(r1)),
                            GenericResource::BasicResources(BasicResource::Oxygen(r2)),
                        ),
                        ComplexResourceRequest::AIPartner(r1, r2) => (
                            GenericResource::ComplexResources(ComplexResource::Robot(r1)),
                            GenericResource::ComplexResources(ComplexResource::Diamond(r2)),
                        ),
                        ComplexResourceRequest::Life(r1, r2) => (
                            GenericResource::ComplexResources(ComplexResource::Water(r1)),
                            GenericResource::BasicResources(BasicResource::Carbon(r2)),
                        ),
                        ComplexResourceRequest::Diamond(r1, r2) => (
                            GenericResource::BasicResources(BasicResource::Carbon(r1)),
                            GenericResource::BasicResources(BasicResource::Carbon(r2)),
                        ),
                        ComplexResourceRequest::Dolphin(r1, r2) => (
                            GenericResource::ComplexResources(ComplexResource::Water(r1)),
                            GenericResource::ComplexResources(ComplexResource::Life(r2)),
                        ),
                        ComplexResourceRequest::Robot(r1, r2) => (
                            GenericResource::BasicResources(BasicResource::Silicon(r1)),
                            GenericResource::ComplexResources(ComplexResource::Life(r2)),
                        ),
                    };

                    res = Some(PlanetToExplorer::CombineResourceResponse {
                        complex_response: Err(("no available cell".to_string(), ret1, ret2)),
                    });
                }

                //LOG
                log_explorer_to_planet!(
                    self,
                    explorer_id,
                    "handle_explorer_msg()";
                    "state"=>format!("{:?}",PlanetState::to_dummy(state)),
                    "_generator"=>"&Generator",
                    "_combinator"=>"&Combinator",
                    "msg" => appo;
                    result = result_str
                );
                //LOG

                res
            }
        }
    }

    fn on_explorer_arrival(
        &mut self,
        _state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
        _explorer_id: ID,
    ) {
        //TODO
    }

    fn on_explorer_departure(
        &mut self,
        _state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
        _explorer_id: ID,
    ) {
        //TODO
    }

    fn on_start(&mut self, state: &PlanetState, _generator: &Generator, _combinator: &Combinator) {
        //println!("Planet {} AI started", state.id());
        //LOG
        log_orch_to_planet!(
            self,
            "on_start()";
            "state"=>format!("{:?}",PlanetState::to_dummy(state)),
            "_generator"=>"&Generator",
            "_combinator"=>"&Combinator",
        );
        //LOG
        // todo!()
    }

    fn on_stop(&mut self, state: &PlanetState, _generator: &Generator, _combinator: &Combinator) {
        //LOG
        log_orch_to_planet!(
            self,
            "on_start()";
            "state"=>format!("{:?}",PlanetState::to_dummy(state)),
            "_generator"=>"&Generator",
            "_combinator"=>"&Combinator",
        );
        //LOG
        // todo!()
    }
}

pub trait ToString2 {
    fn to_string_2(&self) -> String;
}

impl ToString2 for BasicResourceType {
    fn to_string_2(&self) -> String {
        match self {
            BasicResourceType::Carbon => String::from("carbon"),
            BasicResourceType::Hydrogen => String::from("hydrogen"),
            BasicResourceType::Oxygen => String::from("oxygen"),
            BasicResourceType::Silicon => String::from("silicon"),
        }
    }
}
impl ToString2 for ComplexResourceType {
    fn to_string_2(&self) -> String {
        match self {
            ComplexResourceType::AIPartner => String::from("AIPartner"),
            ComplexResourceType::Diamond => String::from("Diamond"),
            ComplexResourceType::Life => String::from("Life"),
            ComplexResourceType::Robot => String::from("Robot"),
            ComplexResourceType::Water => String::from("Water"),
            ComplexResourceType::Dolphin => String::from("Dolphin"),
        }
    }
}

pub const N_CELLS: usize = 5;

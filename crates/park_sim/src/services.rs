//! Deterministic public-path services and conserved guest purchases.
use std::collections::{BTreeMap, BTreeSet};

use crate::{
    Guest, GuestId, GuestState, GuestThought, ObjectId, ObjectKind, Park, PathDirection, PathSlope,
    PlacementError, SurfaceObject, TilePos,
};

pub const MAX_SERVICES: usize = 256;
pub const MAX_SERVICE_PRICE_CENTS: u32 = 10_000;
pub const SERVICE_PRICE_STEP_CENTS: u32 = 50;
pub const SERVICE_USE_TICKS: u16 = 90;
pub const SERVICE_WAIT_TICKS: u16 = 900;
pub const SERVICE_COOLDOWN_TICKS: u16 = 300;
pub const SERVICE_REFUND_PERCENT: u8 = 50;
pub const NEED_SEEK_THRESHOLD: u8 = 60;
pub const NEED_CRITICAL_THRESHOLD: u8 = 85;
// A per-person and per-service bound keeps every bounded save-wide sum safe.
pub(crate) const MAX_SERVICE_USES: u64 = u64::MAX / 256;

#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub struct ServiceId(pub u64);

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum ServiceKind {
    Food,
    Drink,
    Toilet,
    Rest,
    IceCream,
    Souvenir,
    Information,
    FirstAid,
}
impl ServiceKind {
    pub const ALL: [Self; 8] = [
        Self::Food,
        Self::Drink,
        Self::Toilet,
        Self::Rest,
        Self::IceCream,
        Self::Souvenir,
        Self::Information,
        Self::FirstAid,
    ];
    const BASIC_NEEDS: [Self; 3] = [Self::Food, Self::Drink, Self::Toilet];
    pub const fn build_cost_cents(self) -> u64 {
        match self {
            Self::Food => 12_000,
            Self::Drink => 8_000,
            Self::Toilet => 10_000,
            Self::Rest => 14_000,
            Self::IceCream => 11_000,
            Self::Souvenir => 16_000,
            Self::Information => 9_000,
            Self::FirstAid => 18_000,
        }
    }
    pub const fn upkeep_cost_cents(self) -> u32 {
        match self {
            Self::Food => 30,
            Self::Drink => 20,
            Self::Toilet => 10,
            Self::Rest => 25,
            Self::IceCream => 25,
            Self::Souvenir => 35,
            Self::Information => 15,
            Self::FirstAid => 40,
        }
    }
    pub const fn supply_cost_cents(self) -> u32 {
        match self {
            Self::Food => 60,
            Self::Drink => 25,
            Self::Toilet => 10,
            Self::Rest => 20,
            Self::IceCream => 40,
            Self::Souvenir => 150,
            Self::Information => 5,
            Self::FirstAid => 30,
        }
    }
    pub const fn default_price_cents(self) -> u32 {
        match self {
            Self::Food => 200,
            Self::Drink => 100,
            Self::Toilet => 0,
            Self::Rest => 0,
            Self::IceCream => 150,
            Self::Souvenir => 400,
            Self::Information => 0,
            Self::FirstAid => 0,
        }
    }
    fn cadence(self) -> u64 {
        match self {
            Self::Food => 90,
            Self::Drink => 60,
            Self::Toilet => 120,
            Self::Rest => 120,
            Self::IceCream | Self::Souvenir | Self::Information | Self::FirstAid => 120,
        }
    }
    fn need_thought(self) -> GuestThought {
        match self {
            Self::Food => GuestThought::Hungry,
            Self::Drink => GuestThought::Thirsty,
            Self::Toilet => GuestThought::NeedsToilet,
            Self::Rest => GuestThought::NeedsRest,
            Self::IceCream => GuestThought::Thirsty,
            Self::Souvenir => GuestThought::WantsSouvenir,
            Self::Information => GuestThought::NeedsInformation,
            Self::FirstAid => GuestThought::FeelingSick,
        }
    }
    fn relief_thought(self) -> GuestThought {
        match self {
            Self::Food => GuestThought::AteFood,
            Self::Drink => GuestThought::HadDrink,
            Self::Toilet => GuestThought::UsedToilet,
            Self::Rest => GuestThought::Rested,
            Self::IceCream => GuestThought::HadIceCream,
            Self::Souvenir => GuestThought::BoughtSouvenir,
            Self::Information => GuestThought::FoundInformation,
            Self::FirstAid => GuestThought::ReceivedCare,
        }
    }
}
impl std::fmt::Display for ServiceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Food => "Food",
            Self::Drink => "Drinks",
            Self::Toilet => "Toilets",
            Self::Rest => "Rest",
            Self::IceCream => "Ice cream",
            Self::Souvenir => "Souvenirs",
            Self::Information => "Information",
            Self::FirstAid => "First aid",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GuestNeeds {
    #[serde(default, skip_serializing_if = "need_zero")]
    pub souvenir: u8,
    #[serde(default, skip_serializing_if = "need_zero")]
    pub information: u8,
    pub hunger: u8,
    pub thirst: u8,
    pub toilet: u8,
    #[serde(default = "default_energy")]
    pub energy: u8,
    #[serde(default)]
    pub nausea: u8,
}
fn need_zero(value: &u8) -> bool {
    *value == 0
}
const fn default_energy() -> u8 {
    80
}
impl Default for GuestNeeds {
    fn default() -> Self {
        Self {
            souvenir: 0,
            information: 0,
            hunger: 20,
            thirst: 20,
            toilet: 20,
            energy: default_energy(),
            nausea: 0,
        }
    }
}
impl GuestNeeds {
    pub fn for_guest(seed: u64, id: GuestId) -> Self {
        let mut state = seed ^ id.0.wrapping_mul(0xd6e8_feb8_6659_fd93) ^ 0x6e65_6564_7373_6565;
        let mut sample = || 10 + (crate::crowds::next_random(&mut state) % 21) as u8;
        Self {
            souvenir: 0,
            information: 0,
            hunger: sample(),
            thirst: sample(),
            toilet: sample(),
            energy: 70 + sample() / 2,
            nausea: sample() / 3,
        }
    }
    pub(crate) fn for_legacy_guest(seed: u64, id: GuestId) -> Self {
        Self {
            energy: default_energy(),
            nausea: 0,
            ..Self::for_guest(seed, id)
        }
    }
    pub fn value(self, kind: ServiceKind) -> u8 {
        match kind {
            ServiceKind::Food => self.hunger,
            ServiceKind::Drink => self.thirst,
            ServiceKind::Toilet => self.toilet,
            ServiceKind::Rest => 100_u8.saturating_sub(self.energy).max(self.nausea),
            ServiceKind::IceCream => self.hunger.min(self.thirst),
            ServiceKind::Souvenir => self.souvenir,
            ServiceKind::Information => self.information,
            ServiceKind::FirstAid => self.nausea,
        }
    }
    fn value_mut(&mut self, kind: ServiceKind) -> &mut u8 {
        match kind {
            ServiceKind::Food => &mut self.hunger,
            ServiceKind::Drink => &mut self.thirst,
            ServiceKind::Toilet => &mut self.toilet,
            ServiceKind::Souvenir => &mut self.souvenir,
            ServiceKind::Information => &mut self.information,
            ServiceKind::FirstAid => &mut self.nausea,
            ServiceKind::IceCream | ServiceKind::Rest => {
                unreachable!("rest treats both energy and nausea")
            }
        }
    }
    pub(crate) fn relieve(&mut self, kind: ServiceKind) {
        match kind {
            ServiceKind::IceCream => {
                self.hunger = self.hunger.saturating_sub(45);
                self.thirst = self.thirst.saturating_sub(45);
            }
            ServiceKind::Souvenir => self.souvenir = 0,
            ServiceKind::Information => self.information = 0,
            ServiceKind::FirstAid => {
                self.nausea = 0;
                self.energy = self.energy.max(65);
            }
            ServiceKind::Rest => {
                self.energy = self.energy.max(90);
                self.nausea = self.nausea.min(10);
            }
            _ => *self.value_mut(kind) = 20,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Service {
    pub id: ServiceId,
    #[serde(deserialize_with = "crate::saves::required_option")]
    pub object: Option<ObjectId>,
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub kind: ServiceKind,
    /// Outward from the building toward its public serving path.
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub facing: PathDirection,
    pub open: bool,
    pub price_cents: u32,
    pub served: u64,
    pub revenue_cents: u64,
    pub unaffordable_rejections: u64,
    pub upkeep_progress: u16,
    pub upkeep_cents: u64,
    pub supplies_cents: u64,
    pub closed_for_funds: bool,
    /// Actual construction charge; imported free facilities retain zero.
    pub construction_cost_cents: u64,
    /// One-time demolition refund retained with the service's history.
    pub demolition_refund_cents: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ServiceVisitPhase {
    Approaching,
    Waiting,
    Using,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceVisit {
    pub service: ServiceId,
    pub access: ObjectId,
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub phase: ServiceVisitPhase,
    /// A locked quote only during Using; zero before that point.
    pub price_cents: u32,
    pub progress: u16,
    pub waited_ticks: u16,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ServiceAccessStatus {
    Ready,
    Closed,
    Disconnected,
    Removed,
    #[default]
    UnknownService,
}
impl std::fmt::Display for ServiceAccessStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Ready => "Open",
            Self::Closed => "Closed",
            Self::Disconnected => "Connect a public path",
            Self::Removed => "Removed",
            Self::UnknownService => "Unknown service",
        })
    }
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ServiceAccess {
    pub status: ServiceAccessStatus,
    pub public_path: Option<ObjectId>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServiceError {
    UnknownService(ServiceId),
    RemovedService(ServiceId),
    PriceTooHigh { price: u32, max: u32 },
}
impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownService(id) => write!(f, "service #{} does not exist", id.0),
            Self::RemovedService(id) => write!(f, "service #{} has been removed", id.0),
            Self::PriceTooHigh { max, .. } => write!(
                f,
                "service price cannot exceed ${}.{:02}",
                max / 100,
                max % 100
            ),
        }
    }
}
impl std::error::Error for ServiceError {}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ServiceStats {
    pub status: ServiceAccessStatus,
    pub approaching: usize,
    pub waiting: usize,
    pub using: usize,
    pub price_cents: u32,
    pub served: u64,
    pub revenue_cents: u64,
    pub unaffordable_rejections: u64,
    pub upkeep_progress: u16,
    pub upkeep_cents: u64,
    pub supplies_cents: u64,
    pub closed_for_funds: bool,
    pub construction_cost_cents: u64,
    pub demolition_refund_cents: u64,
    pub profit_cents: i64,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GuestNeedsSummary {
    pub guest_count: usize,
    pub hungry_guests: usize,
    pub thirsty_guests: usize,
    pub toilet_guests: usize,
    pub tired_guests: usize,
    pub nauseous_guests: usize,
    pub critical_guests: usize,
    pub seeking: usize,
    pub waiting: usize,
    pub using: usize,
    pub services_used: u64,
    pub service_revenue_cents: u64,
}

impl Park {
    pub fn services(&self) -> &[Service] {
        &self.services
    }
    pub fn service(&self, id: ServiceId) -> Option<&Service> {
        self.services.iter().find(|service| service.id == id)
    }
    pub fn service_by_object(&self, object: ObjectId) -> Option<&Service> {
        self.services
            .iter()
            .find(|service| service.object == Some(object))
    }
    pub fn validate_service(
        &self,
        pos: TilePos,
        height: u8,
        facing: PathDirection,
        kind: ServiceKind,
    ) -> Result<(), PlacementError> {
        self.service_build_quote(pos, height, facing, kind)
            .map(|_| ())
    }
    /// Read-only price and placement validation used again by the command.
    pub fn service_build_quote(
        &self,
        pos: TilePos,
        height: u8,
        _facing: PathDirection,
        kind: ServiceKind,
    ) -> Result<u64, PlacementError> {
        if !self.facility_content_available(kind) {
            return Err(PlacementError::ResearchRequired);
        }
        if !self.service_researched(kind) {
            return Err(PlacementError::ResearchRequired);
        }
        if self.services.len() >= MAX_SERVICES {
            return Err(PlacementError::CapacityExceeded {
                resource: "service records",
                max: MAX_SERVICES,
            });
        }
        if self.objects.len() >= crate::saves::MAX_OBJECTS {
            return Err(PlacementError::CapacityExceeded {
                resource: "objects",
                max: crate::saves::MAX_OBJECTS,
            });
        }
        self.next_service_id
            .checked_add(1)
            .ok_or(PlacementError::IdExhausted)?;
        self.validate_object_placement(pos, ObjectKind::Service, height, 3)?;
        let cost = kind.build_cost_cents();
        if self.cash_cents < cost {
            return Err(PlacementError::InsufficientFunds {
                required_cents: cost,
                available_cents: self.cash_cents,
            });
        }
        Ok(cost)
    }
    /// Unfinished service users can leave safely on the serving public path;
    /// demolition cancels their purchase without charging or granting relief.
    pub fn service_demolition_quote(&self, id: ServiceId) -> Result<u64, ServiceError> {
        let service = self.service(id).ok_or(ServiceError::UnknownService(id))?;
        if service.object.is_none() {
            return Err(ServiceError::RemovedService(id));
        }
        Ok(service.construction_cost_cents * u64::from(SERVICE_REFUND_PERCENT) / 100)
    }
    pub fn place_service(
        &mut self,
        pos: TilePos,
        height: u8,
        facing: PathDirection,
        kind: ServiceKind,
    ) -> Result<ServiceId, PlacementError> {
        let cost = self.service_build_quote(pos, height, facing, kind)?;
        let object = self.insert_surface_object(
            SurfaceObject {
                id: ObjectId(0),
                pos,
                kind: ObjectKind::Service,
                axis: facing.axis(),
                path_slope: PathSlope::Flat,
                queue_owner: None,
                height,
                clearance: 3,
            },
            self.next_object_id + 1,
        );
        let id = ServiceId(self.next_service_id);
        self.next_service_id += 1;
        self.services.push(Service {
            id,
            object: Some(object),
            kind,
            facing,
            open: true,
            price_cents: kind.default_price_cents(),
            served: 0,
            revenue_cents: 0,
            unaffordable_rejections: 0,
            upkeep_progress: 0,
            upkeep_cents: 0,
            supplies_cents: 0,
            closed_for_funds: false,
            construction_cost_cents: cost,
            demolition_refund_cents: 0,
        });
        self.record_cash(crate::FinanceKind::ServiceConstruction, Some(id), cost);
        self.sync_facility_stocks();
        Ok(id)
    }
    pub fn service_access(&self, id: ServiceId) -> ServiceAccess {
        let Some(service) = self.service(id) else {
            return ServiceAccess::default();
        };
        let Some(object) = service.object.and_then(|id| self.object(id)) else {
            return ServiceAccess {
                status: ServiceAccessStatus::Removed,
                public_path: None,
            };
        };
        let public_path = service.facing.neighbor(object.pos).and_then(|pos| {
            self.objects
                .iter()
                .find(|path| {
                    path.is_public_path()
                        && path.pos == pos
                        && path.path_edge_height(service.facing.opposite()) == Some(object.height)
                })
                .map(|path| path.id)
        });
        ServiceAccess {
            status: if !service.open {
                ServiceAccessStatus::Closed
            } else if public_path.is_none() {
                ServiceAccessStatus::Disconnected
            } else {
                ServiceAccessStatus::Ready
            },
            public_path,
        }
    }
    pub(crate) fn service_connects(&self, building: &SurfaceObject, path: &SurfaceObject) -> bool {
        if building.kind != ObjectKind::Service || !path.is_public_path() {
            return false;
        }
        self.service_by_object(building.id).is_some_and(|service| {
            service.facing.neighbor(building.pos) == Some(path.pos)
                && path.path_edge_height(service.facing.opposite()) == Some(building.height)
        })
    }
    pub fn set_service_price(&mut self, id: ServiceId, cents: u32) -> Result<(), ServiceError> {
        let service = self
            .services
            .iter_mut()
            .find(|service| service.id == id)
            .ok_or(ServiceError::UnknownService(id))?;
        if service.object.is_none() {
            return Err(ServiceError::RemovedService(id));
        }
        if cents > MAX_SERVICE_PRICE_CENTS {
            return Err(ServiceError::PriceTooHigh {
                price: cents,
                max: MAX_SERVICE_PRICE_CENTS,
            });
        }
        service.price_cents = cents;
        Ok(())
    }
    pub fn set_service_open(&mut self, id: ServiceId, open: bool) -> Result<(), ServiceError> {
        let service = self
            .services
            .iter_mut()
            .find(|service| service.id == id)
            .ok_or(ServiceError::UnknownService(id))?;
        if service.object.is_none() {
            return Err(ServiceError::RemovedService(id));
        }
        service.open = open;
        service.closed_for_funds = false;
        self.reconcile_service_visits();
        Ok(())
    }
    pub fn service_stats(&self, id: ServiceId) -> ServiceStats {
        let Some(service) = self.service(id) else {
            return ServiceStats::default();
        };
        let mut stats = ServiceStats {
            status: self.service_access(id).status,
            price_cents: service.price_cents,
            served: service.served,
            revenue_cents: service.revenue_cents,
            unaffordable_rejections: service.unaffordable_rejections,
            upkeep_progress: service.upkeep_progress,
            upkeep_cents: service.upkeep_cents,
            supplies_cents: service.supplies_cents,
            closed_for_funds: service.closed_for_funds,
            construction_cost_cents: service.construction_cost_cents,
            demolition_refund_cents: service.demolition_refund_cents,
            profit_cents: service.revenue_cents as i64 + service.demolition_refund_cents as i64
                - service.construction_cost_cents as i64
                - service.upkeep_cents as i64
                - service.supplies_cents as i64,
            ..Default::default()
        };
        for visit in self
            .guests
            .iter()
            .filter_map(|guest| guest.service_visit.as_ref())
            .filter(|visit| visit.service == id)
        {
            match visit.phase {
                ServiceVisitPhase::Approaching => stats.approaching += 1,
                ServiceVisitPhase::Waiting => stats.waiting += 1,
                ServiceVisitPhase::Using => stats.using += 1,
            }
        }
        stats
    }
    pub fn guest_needs_summary(&self) -> GuestNeedsSummary {
        let mut summary = GuestNeedsSummary {
            guest_count: self.guests.len(),
            service_revenue_cents: self
                .services
                .iter()
                .map(|service| service.revenue_cents)
                .sum(),
            ..Default::default()
        };
        for guest in &self.guests {
            summary.hungry_guests += usize::from(guest.needs.hunger >= NEED_SEEK_THRESHOLD);
            summary.thirsty_guests += usize::from(guest.needs.thirst >= NEED_SEEK_THRESHOLD);
            summary.toilet_guests += usize::from(guest.needs.toilet >= NEED_SEEK_THRESHOLD);
            summary.tired_guests += usize::from(guest.needs.energy <= 100 - NEED_SEEK_THRESHOLD);
            summary.nauseous_guests += usize::from(guest.needs.nausea >= NEED_SEEK_THRESHOLD);
            summary.critical_guests += usize::from(
                [
                    ServiceKind::Food,
                    ServiceKind::Drink,
                    ServiceKind::Toilet,
                    ServiceKind::Rest,
                ]
                .iter()
                .any(|kind| guest.needs.value(*kind) >= NEED_CRITICAL_THRESHOLD),
            );
            summary.services_used += guest.services_used;
            if let Some(visit) = &guest.service_visit {
                match visit.phase {
                    ServiceVisitPhase::Approaching => summary.seeking += 1,
                    ServiceVisitPhase::Waiting => summary.waiting += 1,
                    ServiceVisitPhase::Using => summary.using += 1,
                }
            }
        }
        summary
    }

    pub(crate) fn step_needs(&mut self) {
        let benches = &self.decorations.benches;
        for guest in &mut self.guests {
            if self.ticks.is_multiple_of(180) {
                guest.needs.souvenir = guest.needs.souvenir.saturating_add(1).min(100);
            }
            if self.ticks.is_multiple_of(240) {
                guest.needs.information = guest.needs.information.saturating_add(1).min(100);
            }
            for kind in ServiceKind::BASIC_NEEDS {
                if !self.ticks.is_multiple_of(kind.cadence()) {
                    continue;
                }
                let value = guest.needs.value_mut(kind);
                let old = *value;
                *value = value.saturating_add(1).min(100);
                if old < NEED_CRITICAL_THRESHOLD && *value >= NEED_CRITICAL_THRESHOLD {
                    guest.satisfaction = guest.satisfaction.saturating_sub(5);
                    guest.think(kind.need_thought(), self.ticks);
                }
            }
            if self.ticks.is_multiple_of(90) {
                let old_energy = guest.needs.energy;
                if guest.state == GuestState::Walking {
                    if guest.current.is_some_and(|id| {
                        benches
                            .binary_search_by_key(&id, |bench| bench.path)
                            .is_ok()
                    }) {
                        guest.needs.energy = guest.needs.energy.saturating_add(2).min(100);
                    } else {
                        guest.needs.energy = guest.needs.energy.saturating_sub(1);
                    }
                } else if self.ticks.is_multiple_of(180) {
                    guest.needs.energy = guest.needs.energy.saturating_add(1).min(100);
                }
                guest.needs.nausea = guest.needs.nausea.saturating_sub(1);
                if old_energy > 15 && guest.needs.energy <= 15 {
                    guest.satisfaction = guest.satisfaction.saturating_sub(5);
                    guest.think(GuestThought::NeedsRest, self.ticks);
                }
            }
            guest.service_cooldown = guest.service_cooldown.saturating_sub(1);
        }
    }
    pub(crate) fn reconcile_service_visits(&mut self) {
        if self
            .guests
            .iter()
            .all(|guest| guest.service_visit.is_none())
        {
            return;
        }
        let access: BTreeMap<_, _> = self
            .services
            .iter()
            .map(|service| (service.id, self.service_access(service.id)))
            .collect();
        for guest in &mut self.guests {
            let Some(visit) = &guest.service_visit else {
                continue;
            };
            if guest.ride_visit.is_some()
                || guest.current.is_none()
                || guest.state == GuestState::Lost
                || access.get(&visit.service).is_none_or(|access| {
                    access.status != ServiceAccessStatus::Ready
                        || access.public_path != Some(visit.access)
                })
            {
                cancel_visit(guest, GuestThought::ServiceCancelled, self.ticks);
            }
        }
    }
    pub(crate) fn step_services(&mut self, graph: &crate::crowds::PathGraph) {
        self.reconcile_service_visits();
        // Finish occupied counters before awarding any newly available slot.
        // Stable service/guest IDs determine the order when limited cash can
        // fund only some completions. Serialized vector order is irrelevant.
        let mut customers: Vec<_> = self
            .guests
            .iter()
            .enumerate()
            .filter_map(|(index, guest)| {
                guest
                    .service_visit
                    .as_ref()
                    .map(|visit| (visit.service, guest.id, index))
            })
            .collect();
        customers.sort_unstable();
        for (_, _, index) in customers {
            let Some(visit) = &mut self.guests[index].service_visit else {
                continue;
            };
            if visit.phase == ServiceVisitPhase::Using {
                visit.progress += 1;
                if visit.progress < SERVICE_USE_TICKS {
                    continue;
                }
                let id = visit.service;
                let price = visit.price_cents;
                if !self.stock_available(id) {
                    self.guests[index].service_visit = None;
                    self.guests[index].service_cooldown = SERVICE_COOLDOWN_TICKS;
                    self.guests[index].think(GuestThought::ServiceUnavailable, self.ticks);
                    continue;
                }
                let prepaid = self.consume_stock(id);
                let supply = if prepaid {
                    0
                } else {
                    u64::from(self.service(id).unwrap().kind.supply_cost_cents())
                };
                if self.cash_cents + u64::from(price) < supply {
                    let service = self
                        .services
                        .iter_mut()
                        .find(|service| service.id == id)
                        .unwrap();
                    service.open = false;
                    service.closed_for_funds = true;
                    self.reconcile_service_visits();
                    continue;
                }
                let service = self
                    .services
                    .iter_mut()
                    .find(|service| service.id == id)
                    .expect("validated live service visit");
                let guest = &mut self.guests[index];
                guest.wallet_cents -= price;
                guest.spent_cents += u64::from(price);
                guest.service_spent_cents += u64::from(price);
                guest.services_used += 1;
                service.served += 1;
                service.revenue_cents += u64::from(price);
                service.supplies_cents += supply;
                guest.needs.relieve(service.kind);
                guest.satisfaction = guest.satisfaction.saturating_add(5).min(100);
                guest.think(service.kind.relief_thought(), self.ticks);
                guest.service_visit = None;
                guest.service_cooldown = SERVICE_COOLDOWN_TICKS;
                guest.wait_ticks = 1;
                self.record_cash(
                    crate::FinanceKind::ServiceRevenue,
                    Some(id),
                    u64::from(price),
                );
                self.record_cash(crate::FinanceKind::ServiceSupplies, Some(id), supply);
                continue;
            }
            if visit.phase == ServiceVisitPhase::Approaching {
                let destination = visit.access;
                let goal = self.visitors.walk_goal(self.guests[index].id);
                graph.step_guest(&mut self.guests[index], &mut self.crowd_rng, goal);
                let guest = &mut self.guests[index];
                if guest.next.is_none() && guest.current == Some(destination) {
                    guest.wait_ticks = 0;
                    guest.destination = None;
                    guest.route.clear();
                    guest.state = GuestState::Waiting;
                    guest.service_visit.as_mut().unwrap().phase = ServiceVisitPhase::Waiting;
                    guest.think(GuestThought::WaitingForService, self.ticks);
                }
            }
        }
        let access: BTreeMap<_, _> = self
            .services
            .iter()
            .map(|service| (service.id, self.service_access(service.id)))
            .collect();
        let stocked: BTreeSet<_> = self
            .services
            .iter()
            .filter(|s| self.stock_available(s.id))
            .map(|s| s.id)
            .collect();
        // A failed search is bounded by cooldown, with no navigation RNG draw.
        for guest in &mut self.guests {
            if self.visitors.leaving(guest.id)
                || guest.ride_visit.is_some()
                || guest.service_visit.is_some()
                || guest.service_cooldown != 0
                || guest.next.is_some()
                || guest.wait_ticks != 0
                || guest.current.is_none()
            {
                continue;
            }
            if !ServiceKind::ALL
                .iter()
                .any(|kind| guest.needs.value(*kind) >= NEED_SEEK_THRESHOLD)
            {
                continue;
            }
            let current = guest.current.unwrap();
            let mut best = None;
            let mut unaffordable = None;
            for service in &self.services {
                let need = guest.needs.value(service.kind);
                let serving = &access[&service.id];
                if need < NEED_SEEK_THRESHOLD
                    || !stocked.contains(&service.id)
                    || serving.status != ServiceAccessStatus::Ready
                    || service.served >= MAX_SERVICE_USES
                    || guest.services_used >= MAX_SERVICE_USES
                {
                    continue;
                }
                let Some(path) = serving.public_path else {
                    continue;
                };
                let Some(route) = graph.route(current, path) else {
                    continue;
                };
                let key = (100 - need, service.kind, route.len(), service.id);
                if guest.wallet_cents < service.price_cents {
                    if unaffordable.as_ref().is_none_or(|(old, _)| key < *old) {
                        unaffordable = Some((key, service.id));
                    }
                } else if best.as_ref().is_none_or(|(old, _, _, _)| key < *old) {
                    best = Some((key, service.id, path, route));
                }
            }
            if let Some((_, service, path, mut route)) = best {
                guest.next = route.pop_front();
                guest.destination = guest.next.map(|_| path);
                guest.route = route;
                guest.progress = 0;
                guest.state = if guest.next.is_some() {
                    GuestState::Walking
                } else {
                    GuestState::Waiting
                };
                let phase = if guest.next.is_some() {
                    ServiceVisitPhase::Approaching
                } else {
                    ServiceVisitPhase::Waiting
                };
                guest.service_visit = Some(ServiceVisit {
                    service,
                    access: path,
                    phase,
                    price_cents: 0,
                    progress: 0,
                    waited_ticks: 0,
                });
                guest.think(
                    if phase == ServiceVisitPhase::Approaching {
                        GuestThought::SeekingService
                    } else {
                        GuestThought::WaitingForService
                    },
                    self.ticks,
                );
            } else {
                guest.service_cooldown = SERVICE_COOLDOWN_TICKS;
                guest.wait_ticks = 1;
                guest.think(
                    if unaffordable.is_some() {
                        GuestThought::CannotAffordService
                    } else {
                        GuestThought::ServiceUnavailable
                    },
                    self.ticks,
                );
                if let Some((_, id)) = unaffordable {
                    let service = self
                        .services
                        .iter_mut()
                        .find(|service| service.id == id)
                        .unwrap();
                    service.unaffordable_rejections =
                        service.unaffordable_rejections.saturating_add(1);
                }
            }
        }
        let mut occupied: BTreeSet<_> = self
            .guests
            .iter()
            .filter_map(|guest| {
                guest
                    .service_visit
                    .as_ref()
                    .filter(|visit| visit.phase == ServiceVisitPhase::Using)
                    .map(|visit| visit.service)
            })
            .collect();
        let mut waiting: Vec<_> = self
            .guests
            .iter()
            .enumerate()
            .filter(|(_, guest)| {
                guest
                    .service_visit
                    .as_ref()
                    .is_some_and(|visit| visit.phase == ServiceVisitPhase::Waiting)
            })
            .map(|(index, guest)| (guest.id, index))
            .collect();
        waiting.sort_unstable();
        for (_, index) in waiting {
            let guest = &mut self.guests[index];
            let visit = guest.service_visit.as_mut().unwrap();
            visit.waited_ticks += 1;
            if visit.waited_ticks >= SERVICE_WAIT_TICKS {
                cancel_visit(guest, GuestThought::ServiceUnavailable, self.ticks);
                continue;
            }
            if occupied.contains(&visit.service) {
                continue;
            }
            let service = self
                .services
                .iter_mut()
                .find(|service| service.id == visit.service)
                .unwrap();
            if guest.wallet_cents < service.price_cents {
                service.unaffordable_rejections = service.unaffordable_rejections.saturating_add(1);
                cancel_visit(guest, GuestThought::CannotAffordService, self.ticks);
            } else if guest.services_used >= MAX_SERVICE_USES || service.served >= MAX_SERVICE_USES
            {
                cancel_visit(guest, GuestThought::ServiceUnavailable, self.ticks);
            } else {
                visit.phase = ServiceVisitPhase::Using;
                visit.price_cents = service.price_cents;
                visit.progress = 0;
                occupied.insert(visit.service);
                guest.think(GuestThought::UsingService, self.ticks);
            }
        }
    }
}

fn cancel_visit(guest: &mut Guest, thought: GuestThought, tick: u64) {
    guest.service_visit = None;
    guest.service_cooldown = SERVICE_COOLDOWN_TICKS;
    guest.route.clear();
    guest.destination = guest.next;
    if guest.next.is_none() && guest.current.is_some() {
        guest.state = GuestState::Waiting;
        guest.wait_ticks = 1;
    }
    guest.think(thought, tick);
}

pub(crate) fn validate_services(park: &Park) -> Result<(), String> {
    if park.services.len() > MAX_SERVICES || park.next_service_id == 0 {
        return Err("service collection or allocator is invalid".into());
    }
    let mut ids = BTreeMap::new();
    let mut objects = BTreeSet::new();
    let mut served = 0_u64;
    let mut revenue = 0_u64;
    for service in &park.services {
        if (service.construction_cost_cents != 0
            && service.construction_cost_cents != service.kind.build_cost_cents())
            || service.demolition_refund_cents
                != if service.object.is_none() {
                    service.construction_cost_cents * u64::from(SERVICE_REFUND_PERCENT) / 100
                } else {
                    0
                }
        {
            return Err("service construction basis or demolition refund is invalid".into());
        }
        if service.id.0 == 0
            || service.id.0 >= park.next_service_id
            || ids.insert(service.id, service).is_some()
            || service.price_cents > MAX_SERVICE_PRICE_CENTS
            || service.served > MAX_SERVICE_USES
            || u128::from(service.revenue_cents)
                > u128::from(service.served) * u128::from(crate::INITIAL_GUEST_WALLET_CENTS)
            || (service.object.is_none() && service.open)
        {
            return Err("service identity, price or history is invalid".into());
        }
        if let Some(id) = service.object {
            let object = park.object(id).ok_or("service object is missing")?;
            if !objects.insert(id)
                || object.kind != ObjectKind::Service
                || object.clearance != 3
                || object.path_slope != PathSlope::Flat
                || object.queue_owner.is_some()
                || object.axis != service.facing.axis()
            {
                return Err("service object metadata is inconsistent".into());
            }
        }
        served = served
            .checked_add(service.served)
            .ok_or("service use totals overflow")?;
        revenue = revenue
            .checked_add(service.revenue_cents)
            .ok_or("service revenue overflows")?;
    }
    if park
        .objects
        .iter()
        .any(|object| object.kind == ObjectKind::Service && !objects.contains(&object.id))
    {
        return Err("service object has no metadata".into());
    }
    let mut users = BTreeSet::new();
    let mut guest_served = park.visitors.departed.services_used;
    let mut guest_revenue = park.visitors.departed.service_spent_cents;
    for guest in &park.guests {
        if ServiceKind::ALL
            .iter()
            .any(|kind| guest.needs.value(*kind) > 100)
            || guest.needs.energy > 100
            || guest.needs.nausea > 100
            || guest.service_cooldown > SERVICE_COOLDOWN_TICKS
            || guest.services_used > MAX_SERVICE_USES
            || guest.service_spent_cents > guest.spent_cents
            || (guest.services_used == 0 && guest.service_spent_cents != 0)
        {
            return Err("guest needs or service history is invalid".into());
        }
        guest_served = guest_served
            .checked_add(guest.services_used)
            .ok_or("guest service counts overflow")?;
        guest_revenue = guest_revenue
            .checked_add(guest.service_spent_cents)
            .ok_or("guest service spending overflows")?;
        let Some(visit) = &guest.service_visit else {
            continue;
        };
        let service = ids
            .get(&visit.service)
            .ok_or("guest service visit references no service")?;
        let access = park.service_access(visit.service);
        if guest.ride_visit.is_some()
            || guest.service_cooldown != 0
            || (matches!(
                service.kind,
                ServiceKind::Food | ServiceKind::Drink | ServiceKind::Toilet
            ) && guest.needs.value(service.kind) < NEED_SEEK_THRESHOLD)
            || guest.current.is_none()
            || guest.state == GuestState::Lost
            || access.status != ServiceAccessStatus::Ready
            || access.public_path != Some(visit.access)
            || visit.waited_ticks >= SERVICE_WAIT_TICKS
            || guest.wait_ticks != 0
        {
            return Err("guest service visit conflicts with access or navigation".into());
        }
        match visit.phase {
            ServiceVisitPhase::Approaching => {
                if visit.price_cents != 0
                    || visit.progress != 0
                    || visit.waited_ticks != 0
                    || guest.state != GuestState::Walking
                    || guest.next.is_none()
                    || guest.destination != Some(visit.access)
                {
                    return Err("service approach route is inconsistent".into());
                }
            }
            ServiceVisitPhase::Waiting | ServiceVisitPhase::Using => {
                if guest.current != Some(visit.access)
                    || guest.next.is_some()
                    || guest.destination.is_some()
                    || !guest.route.is_empty()
                    || guest.progress != 0
                    || guest.state != GuestState::Waiting
                {
                    return Err("service customer is not at the public counter".into());
                }
                if visit.phase == ServiceVisitPhase::Waiting {
                    if visit.price_cents != 0 || visit.progress != 0 {
                        return Err("waiting service visit contains a locked transaction".into());
                    }
                } else if !users.insert(visit.service)
                    || visit.progress >= SERVICE_USE_TICKS
                    || visit.price_cents > MAX_SERVICE_PRICE_CENTS
                    || visit.price_cents > guest.wallet_cents
                    || guest.services_used >= MAX_SERVICE_USES
                    || service.served >= MAX_SERVICE_USES
                {
                    return Err("service use quote, capacity or reserved history is invalid".into());
                }
            }
        }
    }
    if guest_served != served || guest_revenue != revenue {
        return Err("service history disagrees with guest purchases".into());
    }
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests;

#[cfg(test)]
mod construction_tests;

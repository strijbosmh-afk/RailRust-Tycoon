//! Natural reviews, derived choice inputs and matched-layout choice evidence.
use super::*;
use park_sim::{GuestId, RideId, RideKind, RideOperatingSettings, RideVisitPhase, TrackKind};

const REPLAY_TICKS: u64 = 7_200;

fn require(value: bool, message: &str) -> Result<(), String> {
    if value { Ok(()) } else { Err(message.into()) }
}

pub(crate) fn install(park: &mut Park) -> Result<[RideId; 2], String> {
    let (coaster, carousel) = super::ride_choice::install(park)?;
    require(
        park.all_rides()
            .all(|r| r.rating_count == 0 && r.revenue_cents == 0),
        "appeal fixture must start with real unrated rides",
    )?;
    Ok([coaster, carousel])
}

/// Independently reconcile the published formula against installed geometry,
/// owned reviews and guest preferences, then prove every query is read-only.
pub(crate) fn inspect(park: &Park) -> Result<String, String> {
    let original = park.clone();
    let saved = super::ride_choice::bytes(park)?;
    let mut report = String::new();
    for ride in park.all_rides() {
        let profile = park
            .ride_appeal_profile(ride.id)
            .ok_or("missing appeal profile")?;
        let challenge = if park.ride_kind(ride.id) == Some(RideKind::Carousel) {
            20
        } else {
            let specs = park.circuit().ok_or("missing appeal circuit")?.specs();
            let mut drop = 0_u32;
            let mut longest = 0_u32;
            // Doubling covers a drop spanning the ordered circuit seam.
            for piece in specs.iter().cycle().take(specs.len() * 2) {
                let descent = piece.entry.height.saturating_sub(piece.exit.height);
                drop = if descent > 0 {
                    drop + u32::from(descent)
                } else {
                    0
                };
                longest = longest.max(drop);
            }
            let density =
                specs.iter().filter(|p| p.kind == TrackKind::Curve).count() * 100 / specs.len();
            (longest.min(12) * 5 + density as u32 * 2 / 5).min(100) as u8
        };
        let reputation = ((u128::from(ride.rating_total) * 10 + 150)
            / (u128::from(ride.rating_count) + 5)) as u8;
        require(
            profile.layout_challenge == challenge
                && profile.review_count == ride.rating_count
                && profile.reputation_rating_tenths == reputation
                && profile.reputation_penalty == 50 - reputation,
            "published appeal differs from installed geometry or independently owned reviews",
        )?;
        report.push_str(&format!("Ride {}: {profile:?}\n", ride.id.0));
        for guest in park.guests() {
            let fit = park
                .guest_ride_appeal(guest.id, ride.id)
                .ok_or("missing guest appeal")?;
            let preference = park
                .guest_preferred_challenge(guest.id)
                .ok_or("missing preference")?;
            require(
                [25, 45, 65, 85].contains(&preference)
                    && fit.profile == profile
                    && fit.preferred_challenge == preference
                    && fit.layout_penalty == u16::from(challenge.saturating_sub(preference)) * 2
                    && fit.total_penalty == fit.layout_penalty + u16::from(50 - reputation)
                    && park.guest_ride_appeal(guest.id, ride.id) == Some(fit),
                "guest appeal differs from shared preference inputs",
            )?;
            report.push_str(&format!("Guest {}: {fit:?}\n", guest.id.0));
        }
    }
    require(
        park.ride_appeal_profile(RideId(u64::MAX)).is_none()
            && park.guest_preferred_challenge(GuestId(u64::MAX)).is_none(),
        "unknown owner or guest fabricated an appeal profile",
    )?;
    require(
        *park == original && park.to_save_bytes().map_err(|e| e.to_string())? == saved,
        "appeal queries changed park, RNG or save bytes",
    )?;
    Ok(report)
}

pub(crate) fn earn_reviews(park: &mut Park, owners: [RideId; 2]) -> Result<GuestId, String> {
    super::ride_choice::seek(
        park,
        |p| {
            owners.iter().all(|id| {
                p.ride_by_id(*id)
                    .is_some_and(|r| r.rating_count > 0 && r.completed > 0 && r.revenue_cents > 0)
            }) && owners.iter().any(|id| {
                p.ride_appeal_profile(*id)
                    .is_some_and(|profile| profile.reputation_rating_tenths != 30)
            })
        },
        "both rides earn real paid public-return reviews",
    )?;
    // An existing approach cannot establish a fresh decision after feedback.
    super::ride_choice::seek_new_choice(park, owners[0])
}

pub(crate) struct ChoicePair {
    pub graded: Park,
    pub low: Park,
    pub guest: GuestId,
    pub coaster: RideId,
    pub carousel: RideId,
}

fn lower_layout(park: &mut Park) -> Result<(), String> {
    park.copy_installed_track_to_draft()
        .map_err(|e| e.to_string())?;
    for mut spec in park.track_draft().to_vec() {
        // Two lifts retain height 10 above the public path crossing at (13,10).
        // The station, footprint, curves, brakes and all access stay identical.
        if spec.pos.y == 10 && (11..=16).contains(&spec.pos.x) {
            spec.kind = TrackKind::Straight;
            spec.entry.height = 10;
            spec.exit.height = 10;
        } else if spec.pos.y == 4 && (20..=23).contains(&spec.pos.x) {
            spec.kind = if spec.pos.x == 23 {
                TrackKind::Drop
            } else {
                TrackKind::Straight
            };
            spec.entry.height = if spec.pos.x == 23 { 10 } else { 8 };
            spec.exit.height = 8;
        } else if spec.entry.height == 16 && spec.exit.height == 16 {
            spec.entry.height = 10;
            spec.exit.height = 10;
        }
        park.set_track_draft_piece(spec)
            .map_err(|e| e.to_string())?;
    }
    park.apply_track_draft().map_err(|e| e.to_string())
}

/// Find a witnessed first decision difference, never a hand-edited preference
/// or review. Both branches have the same access, prices and pre-choice guests.
pub(crate) fn choice_pair(source: &Park) -> Result<ChoicePair, String> {
    let mut baseline = source.clone();
    let owners = install(&mut baseline)?;
    for id in owners {
        baseline
            .set_ride_open(id, false)
            .map_err(|e| e.to_string())?;
        let settings = baseline
            .ride_operating_settings(id)
            .ok_or("missing settings")?;
        baseline
            .set_ride_operating_settings(
                id,
                RideOperatingSettings {
                    station_wait_ticks: 900,
                    ..settings
                },
            )
            .map_err(|e| e.to_string())?;
    }
    let mut lower = baseline.clone();
    lower_layout(&mut lower)?;
    require(
        lower.guests() == baseline.guests()
            && lower
                .objects()
                .iter()
                .filter(|o| o.kind != ObjectKind::Track)
                .collect::<Vec<_>>()
                == baseline
                    .objects()
                    .iter()
                    .filter(|o| o.kind != ObjectKind::Track)
                    .collect::<Vec<_>>()
            && owners.iter().all(|id| {
                lower.ride_by_id(*id) == baseline.ride_by_id(*id)
                    && lower.ride_access(*id) == baseline.ride_access(*id)
                    && lower.ride_exit_access(*id) == baseline.ride_exit_access(*id)
                    && lower.ride_operating_settings(*id) == baseline.ride_operating_settings(*id)
            }),
        "layout comparison changed guests, access, owners or operating settings",
    )?;
    require(
        lower
            .ride_appeal_profile(owners[0])
            .unwrap()
            .layout_challenge
            < baseline
                .ride_appeal_profile(owners[0])
                .unwrap()
                .layout_challenge,
        "lower installed layout did not reduce static challenge",
    )?;
    inspect(&baseline)?;
    inspect(&lower)?;
    // The fixed crowd contains different route distances and soft preferences.
    // A bounded fare sweep finds a marginal choice without changing either
    // branch's guests or assigning a different fare between the branches.
    for fare in (0..=1_000).step_by(25) {
        for prices in [[fare, 100], [100, fare]] {
            let mut graded = baseline.clone();
            let mut low = lower.clone();
            for park in [&mut graded, &mut low] {
                for (id, price) in owners.into_iter().zip(prices) {
                    park.set_ride_ticket_price(id, price)
                        .map_err(|e| e.to_string())?;
                    park.set_ride_open(id, true).map_err(|e| e.to_string())?;
                }
            }
            for _ in 0..120 {
                require(
                    graded.guests() == low.guests(),
                    "layout branches diverged before compared choice",
                )?;
                let previous = graded.guests().to_vec();
                super::ride_choice::advance_checked(&mut graded)?;
                super::ride_choice::advance_checked(&mut low)?;
                require(
                    owners
                        .iter()
                        .all(|id| !graded.ride_is_running(*id) && !low.ride_is_running(*id)),
                    "layout choice comparison advanced ride motion",
                )?;
                if graded.guests() != low.guests() {
                    let selected = previous.iter().find(|old| {
                        if old.ride_visit.is_some() || prices.iter().any(|price|
                            old.wallet_cents < *price || !old.willing_to_pay(*price)) { return false; }
                        let a = graded.guests().iter().find(|g| g.id == old.id).and_then(|g| g.ride_visit.as_ref());
                        let b = low.guests().iter().find(|g| g.id == old.id).and_then(|g| g.ride_visit.as_ref());
                        matches!((a, b), (Some(a), Some(b)) if a.ride == owners[1] && b.ride == owners[0]
                            && a.phase == RideVisitPhase::Approaching && b.phase == RideVisitPhase::Approaching)
                    }).map(|g| g.id);
                    if let Some(guest) = selected {
                        let high_fit = graded.guest_ride_appeal(guest, owners[0]).unwrap();
                        let low_fit = low.guest_ride_appeal(guest, owners[0]).unwrap();
                        require(
                            high_fit.layout_penalty > low_fit.layout_penalty
                                && high_fit.profile.reputation_penalty
                                    == low_fit.profile.reputation_penalty,
                            "observed choice difference lacks a lower layout preference cost",
                        )?;
                        return Ok(ChoicePair {
                            graded,
                            low,
                            guest,
                            coaster: owners[0],
                            carousel: owners[1],
                        });
                    }
                    break;
                }
            }
        }
    }
    Err("no observed first guest choice changed between matched installed layouts".into())
}

fn pair_report(pair: &ChoicePair) -> Result<String, String> {
    Ok(format!(
        "Guest {} chose carousel {} for graded layout and coaster {} for low layout at tick {}.\nPrices: coaster {}, carousel {}.\nGraded:\n{}Low:\n{}",
        pair.guest.0,
        pair.carousel.0,
        pair.coaster.0,
        pair.graded.ticks(),
        pair.graded
            .ride_by_id(pair.coaster)
            .unwrap()
            .ticket_price_cents,
        pair.graded
            .ride_by_id(pair.carousel)
            .unwrap()
            .ticket_price_cents,
        inspect(&pair.graded)?,
        inspect(&pair.low)?
    ))
}

fn replay(park: &mut Park) -> Result<(), String> {
    let before = park
        .all_rides()
        .map(|r| (r.id, r.completed, r.revenue_cents))
        .collect::<Vec<_>>();
    for tick in 0..REPLAY_TICKS {
        super::ride_choice::advance_checked(park)?;
        if tick % 257 == 0 {
            inspect(park)?;
            *park = Park::from_save_bytes(&super::ride_choice::bytes(park)?)
                .map_err(|e| e.to_string())?;
        }
    }
    require(
        before.iter().all(|(id, completed, revenue)| {
            park.ride_by_id(*id)
                .is_some_and(|r| r.completed > *completed && r.revenue_cents > *revenue)
        }),
        "appeal continuation did not produce more real paid completed visits on both rides",
    )
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let mut park = demonstration_park();
    let owners = install(&mut park)?;
    let mut view = ViewState::default();
    for (name, reviewed) in [("backup", false), ("primary", true)] {
        if reviewed {
            earn_reviews(&mut park, owners)?;
        }
        let report = inspect(&park)?;
        if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
            return Err(view.save_notice);
        }
        write_new(
            &store.directory().join(format!("expected-{name}.park.json")),
            &super::ride_choice::bytes(&park)?,
        )?;
        write_new(
            &store.directory().join(format!("expected-{name}.txt")),
            report.as_bytes(),
        )?;
    }
    replay(&mut park)?;
    write_new(
        &store.directory().join("expected-after.park.json"),
        &super::ride_choice::bytes(&park)?,
    )?;
    let pair = choice_pair(&demonstration_park())?;
    write_new(
        &store.directory().join("expected-choice.txt"),
        pair_report(&pair)?.as_bytes(),
    )?;
    for (name, park) in [("graded", pair.graded), ("low", pair.low)] {
        write_new(
            &store
                .directory()
                .join(format!("expected-choice-{name}.park.json")),
            &super::ride_choice::bytes(&park)?,
        )?;
    }
    Ok(())
}

pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    let primary = fs::read(store.save_path()).map_err(|e| e.to_string())?;
    let backup = fs::read(store.backup_path()).map_err(|e| e.to_string())?;
    let mut park = demonstration_park();
    let mut view = ViewState::default();
    for (name, action) in [
        ("primary", SaveAction::Load),
        ("backup", SaveAction::Backup),
    ] {
        if !persistence::perform_action(session, &mut park, &mut view, action) {
            return Err(view.save_notice);
        }
        require(
            view.speed == Speed::Paused
                && park
                    == read_expected(
                        &store.directory().join(format!("expected-{name}.park.json")),
                    )?,
            "appeal slot did not restore the exact paused state",
        )?;
        require(
            inspect(&park)?.as_bytes()
                == fs::read(store.directory().join(format!("expected-{name}.txt")))
                    .map_err(|e| e.to_string())?,
            "fresh-process appeal inputs differ",
        )?;
        if name == "primary" {
            replay(&mut park)?;
            require(
                park == read_expected(&store.directory().join("expected-after.park.json"))?,
                "appeal continuation diverged",
            )?;
        } else {
            let owners = [park.ride().unwrap().id, park.carousels()[0].ride.id];
            earn_reviews(&mut park, owners)?;
            require(
                park == read_expected(&store.directory().join("expected-primary.park.json"))?,
                "unrated backup did not reproduce earned reviews and fresh choice",
            )?;
        }
    }
    let pair = choice_pair(&demonstration_park())?;
    require(
        pair_report(&pair)?.as_bytes()
            == fs::read(store.directory().join("expected-choice.txt"))
                .map_err(|e| e.to_string())?
            && pair.graded
                == read_expected(&store.directory().join("expected-choice-graded.park.json"))?
            && pair.low == read_expected(&store.directory().join("expected-choice-low.park.json"))?,
        "fresh-process causal layout choice differed",
    )?;
    require(
        fs::read(store.save_path()).map_err(|e| e.to_string())? == primary
            && fs::read(store.backup_path()).map_err(|e| e.to_string())? == backup,
        "appeal proof changed source slots",
    )?;
    write_new(&store.directory().join("verification-appeal.txt"), format!(
        "Parkbound {}\nUnrated backup and naturally earned paid reviews in primary: PASS\nNew guest choice after actual public-return reviews: PASS\nShared appeal formulas and repeated query purity including RNG/save bytes: PASS\nObserved first-choice change between matched installed layouts before ride motion: PASS\nExact fresh-process replay: PASS ({REPLAY_TICKS} ticks, owner-specific fares, guest histories, reviews and ledger)\nBackup reproduces reviewed primary and causal layout pair exactly: PASS\nOriginal primary and backup unchanged: PASS\nSoft preferences and cumulative neutral-prior reputation; no guaranteed choice, safety or real-force claim.\nNative pointer gestures: NOT CLAIMED\n", env!("CARGO_PKG_VERSION")).as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn appeal_proof_replays_real_reviews_and_observed_layout_choice_without_changing_slots() {
        let directory = std::env::temp_dir().join(format!(
            "Parkbound appeal proof é {} {}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for mode in ["write-appeal", "verify-appeal"] {
            assert_eq!(
                run(vec![
                    "--save-check".into(),
                    mode.into(),
                    "--save-dir".into(),
                    directory.clone().into_os_string()
                ]),
                Some(AppExit::Success)
            );
        }
        let receipt = fs::read_to_string(directory.join("verification-appeal.txt")).unwrap();
        assert!(receipt.contains("Original primary and backup unchanged: PASS"));
        assert!(receipt.contains("Observed first-choice change"));
        fs::remove_dir_all(directory).unwrap();
    }
}

// Synthetic older-format fixtures use the older cost-free ride rules. This is
// never used by production readers or authentic old-binary migration proofs.
pub(crate) fn project_old_fixture(value: &mut serde_json::Value) {
    use serde_json::json;
    if value["schema_version"].as_u64().is_some_and(|v| v <= 22)
        && let Some(paints) = value["park"].as_object_mut().unwrap().remove("ride_paints")
    {
        assert_eq!(paints, json!([]));
    }
    if value["schema_version"].as_u64().is_some_and(|v| v <= 21)
        && let Some(flow) = value["park"].as_object_mut().unwrap().remove("visitors")
    {
        assert_eq!(
            flow,
            serde_json::json!({"entry":null,"enabled":false,"arrival_progress":0,"spawned":0,"departed":{"count":0,"highest_id":0,"wallet_cents":0,"spent_cents":0,"service_spent_cents":0,"services_used":0,"rides":[]},"stays":[]})
        );
    }
    project_pre_loan(value);
    if !value["schema_version"]
        .as_u64()
        .is_some_and(|v| (1..20).contains(&v))
    {
        return;
    }
    let Some(park) = value.get_mut("park").and_then(|p| p.as_object_mut()) else {
        return;
    };
    let records = park.remove("ride_costs").unwrap_or(json!([]));
    let lifetime = records
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["upkeep_cents"].as_u64().unwrap())
        .sum::<u64>();
    if lifetime != 0 {
        park["cash_cents"] = json!(park["cash_cents"].as_u64().unwrap() + lifetime);
    }
    let coaster = park.get("ride").and_then(|r| r["id"].as_u64());
    let Some(ledger) = park.get_mut("finances").and_then(|f| f.as_object_mut()) else {
        return;
    };
    let recent = ledger["entries"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["kind"] == "RideUpkeep")
        .map(|e| e["amount_cents"].as_u64().unwrap())
        .sum::<u64>();
    let archived_cost = lifetime - recent;
    let archived_count = records
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            let id = r["ride"].as_u64().unwrap();
            let recent_owner = ledger["entries"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| e["kind"] == "RideUpkeep" && e["ride"].as_u64().or(coaster) == Some(id))
                .map(|e| e["amount_cents"].as_u64().unwrap())
                .sum::<u64>();
            (r["upkeep_cents"].as_u64().unwrap() - recent_owner)
                / if Some(id) == coaster { 100 } else { 50 }
        })
        .sum::<u64>();
    ledger["archived"]["upkeep_cents"] =
        json!(ledger["archived"]["upkeep_cents"].as_u64().unwrap() - archived_cost);
    ledger["archived_entries"] =
        json!(ledger["archived_entries"].as_u64().unwrap() - archived_count);
    if ledger["archived_entries"] == 0 {
        ledger["archived_through_tick"] = ledger["opening_tick"].clone();
    }
    for key in ["opening_rides", "archived_rides"] {
        if let Some(rows) = ledger.get_mut(key).and_then(|r| r.as_array_mut()) {
            for row in rows.iter_mut() {
                row.as_object_mut().unwrap().remove("upkeep_cents");
            }
            if key == "archived_rides" {
                rows.retain(|r| {
                    r["revenue_cents"] != 0
                        || r["construction_cents"] != 0
                        || r["demolition_refunds_cents"] != 0
                });
            }
        }
    }
    let mut balance = ledger["opening_cash_cents"].as_u64().unwrap()
        + ledger["archived"]["ride_revenue_cents"].as_u64().unwrap()
        + ledger["archived"]["service_revenue_cents"]
            .as_u64()
            .unwrap()
        + ledger["archived"]
            .get("demolition_refunds_cents")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
        - ledger["archived"]["upkeep_cents"].as_u64().unwrap()
        - ledger["archived"]["supplies_cents"].as_u64().unwrap()
        - ledger["archived"]
            .get("construction_cents")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
    let mut id = ledger["archived_entries"].as_u64().unwrap() + 1;
    let entries = ledger["entries"].as_array_mut().unwrap();
    entries.retain(|e| e["kind"] != "RideUpkeep");
    for entry in entries {
        let amount = entry["amount_cents"].as_u64().unwrap();
        if matches!(
            entry["kind"].as_str().unwrap(),
            "RideRevenue"
                | "ServiceRevenue"
                | "RideDemolitionRefund"
                | "ServiceDemolitionRefund"
                | "TrackRefund"
        ) {
            balance += amount;
        } else {
            balance -= amount;
        }
        entry["id"] = json!(id);
        entry["balance_cents"] = json!(balance);
        id += 1;
    }
    ledger["next_entry_id"] = json!(id);
}

fn project_pre_loan(value: &mut serde_json::Value) {
    if !value["schema_version"]
        .as_u64()
        .is_some_and(|v| (1..=20).contains(&v))
    {
        return;
    }
    let p = value["park"].as_object_mut().unwrap();
    if let Some(a) = p.remove("loans") {
        assert_eq!(
            a,
            serde_json::json!({"principal_cents":0,"borrowed_cents":0,"repaid_cents":0,"interest_accrued_cents":0,"interest_paid_cents":0,"interest_fraction":0})
        );
    }
    if let Some(f) = p
        .get_mut("finances")
        .and_then(|f| f.get_mut("archived"))
        .and_then(|f| f.as_object_mut())
    {
        for k in [
            "loan_borrowed_cents",
            "loan_repaid_cents",
            "loan_interest_cents",
        ] {
            if let Some(v) = f.remove(k) {
                assert_eq!(v, 0);
            }
        }
    }
}

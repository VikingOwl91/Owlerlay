use owlerlay_lib::overlay::model::Layout;
use owlerlay_lib::overlay::service::OverlayService;

#[tokio::test]
async fn create_lists_groups_in_id_order() {
    let svc = OverlayService::new();
    let a = svc.create_group("Intro".into()).await.expect("create");
    let b = svc.create_group("BRB".into()).await.expect("create");

    let groups = svc.list_groups().await;
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].id, a);
    assert_eq!(groups[1].id, b);
    assert_eq!(groups[0].name, "Intro");
}

#[tokio::test]
async fn blank_name_is_rejected() {
    let svc = OverlayService::new();
    assert!(svc.create_group("   ".into()).await.is_err());
}

#[tokio::test]
async fn update_sets_members_layout_and_hide_idle() {
    let svc = OverlayService::new();
    let id = svc.create_group("g".into()).await.expect("create");

    svc.update_group(id, "renamed".into(), vec![3, 1, 2], Layout::Row, true)
        .await
        .expect("update");

    let g = svc.get_group(id).await.expect("group exists");
    assert_eq!(g.name, "renamed");
    assert_eq!(g.members, vec![3, 1, 2]);
    assert_eq!(g.layout, Layout::Row);
    assert!(g.hide_idle);
}

#[tokio::test]
async fn delete_removes_group_and_errors_on_missing() {
    let svc = OverlayService::new();
    let id = svc.create_group("g".into()).await.expect("create");

    svc.delete_group(id).await.expect("delete");
    assert!(svc.get_group(id).await.is_none());
    assert!(svc.delete_group(id).await.is_err());
}

#[tokio::test]
async fn config_round_trips_and_defaults_when_unset() {
    let svc = OverlayService::new();
    // Unset config falls back to defaults so template rendering never sees a hole.
    let default = svc.get_config(42).await;
    assert!(default.show_timer);

    let mut cfg = default.clone();
    cfg.icon = "owl.svg".into();
    svc.set_config(42, cfg).await;
    assert_eq!(svc.get_config(42).await.icon, "owl.svg");
}

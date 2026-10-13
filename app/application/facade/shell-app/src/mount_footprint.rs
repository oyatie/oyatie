#[derive(Default)]
struct MountFootprint {
    generation: u64,
    active: Option<Footprint>,
}

struct Footprint {
    generation: u64,
    original_minimum: InlineMinimum,
    #[cfg(target_arch = "wasm32")]
    host: web_sys::HtmlElement,
}

#[derive(Clone)]
struct InlineMinimum {
    value: String,
    priority: String,
    attribute_present: bool,
}

impl MountFootprint {
    fn begin(
        &mut self,
        original_minimum: InlineMinimum,
        #[cfg(target_arch = "wasm32")] host: web_sys::HtmlElement,
    ) -> Option<u64> {
        let generation = self.generation.checked_add(1)?;
        let original_minimum = self
            .active
            .as_ref()
            .filter(|previous| {
                #[cfg(target_arch = "wasm32")]
                {
                    previous.host == host
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let _ = previous;
                    true
                }
            })
            .map(|previous| previous.original_minimum.clone())
            .unwrap_or(original_minimum);
        self.generation = generation;
        self.active = Some(Footprint {
            generation,
            original_minimum,
            #[cfg(target_arch = "wasm32")]
            host,
        });
        Some(generation)
    }

    fn finish(&mut self, generation: u64, settled: bool) -> Option<Footprint> {
        if settled
            && self
                .active
                .as_ref()
                .is_some_and(|active| active.generation == generation)
        {
            self.active.take()
        } else {
            None
        }
    }
}

fn minimum_height(height: f64) -> Option<f64> {
    (height.is_finite() && height >= 0.0).then(|| height.ceil())
}

fn remove_empty_style_attribute(original_present: bool, remaining_properties: u32) -> bool {
    !original_present && remaining_properties == 0
}

pub(crate) fn replacement_settled(loading: bool, rendered: bool, failed: bool) -> bool {
    !loading && (rendered || failed)
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static FOOTPRINT: std::cell::RefCell<MountFootprint> = const {
        std::cell::RefCell::new(MountFootprint { generation: 0, active: None })
    };
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn protect(host: &web_sys::HtmlElement) -> Option<u64> {
    let height = minimum_height(host.get_bounding_client_rect().height())?;
    FOOTPRINT.with(|footprint| {
        let mut footprint = footprint.borrow_mut();
        footprint.generation.checked_add(1)?;
        let style = host.style();
        let original_minimum = InlineMinimum {
            value: style.get_property_value("min-height").ok()?,
            priority: style.get_property_priority("min-height"),
            attribute_present: host.has_attribute("style"),
        };
        style
            .set_property_with_priority("min-height", &format!("{height}px"), "important")
            .ok()?;
        footprint.begin(original_minimum, host.clone())
    })
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn current_generation() -> Option<u64> {
    FOOTPRINT.with(|footprint| {
        footprint
            .borrow()
            .active
            .as_ref()
            .map(|active| active.generation)
    })
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn release(generation: Option<u64>, settled: bool) {
    let Some(generation) = generation else {
        return;
    };
    FOOTPRINT.with(|footprint| {
        let mut footprint = footprint.borrow_mut();
        let Some(active) = footprint.finish(generation, settled) else {
            return;
        };
        let style = active.host.style();
        let restored = if active.original_minimum.value.is_empty() {
            style.remove_property("min-height").map(|_| ())
        } else {
            style.set_property_with_priority(
                "min-height",
                &active.original_minimum.value,
                &active.original_minimum.priority,
            )
        };
        let restored = restored.and_then(|()| {
            if remove_empty_style_attribute(
                active.original_minimum.attribute_present,
                style.length(),
            ) {
                // Synchronize lazy CSSOM serialization before removing the empty attribute.
                let _ = active.host.get_attribute("style");
                active.host.remove_attribute("style")
            } else {
                Ok(())
            }
        });
        if restored.is_err() {
            footprint.active = Some(active);
        }
    });
}

#[cfg(test)]
fn minimum(value: &str, priority: &str, attribute_present: bool) -> InlineMinimum {
    InlineMinimum {
        value: value.into(),
        priority: priority.into(),
        attribute_present,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_loading_and_pending_bridge_keep_the_measured_footprint() {
        let mut footprint = MountFootprint::default();
        let generation = footprint.begin(minimum("", "", false)).unwrap();
        let protected_height = minimum_height(21_830.656_25).unwrap();
        assert!(protected_height >= 21_830.656_25);
        assert!(
            footprint
                .finish(generation, replacement_settled(true, false, false))
                .is_none()
        );
        // A rendered native sample is not released until its local bridges settle.
        assert!(footprint.finish(generation, false).is_none());
        assert!(
            footprint
                .finish(generation, replacement_settled(false, true, false))
                .is_some()
        );
        assert!(footprint.active.is_none());
    }

    #[test]
    fn superseding_mount_keeps_first_style_and_refuses_stale_completion_or_cleanup() {
        let mut footprint = MountFootprint::default();
        let old = footprint
            .begin(minimum("12rem", "important", true))
            .unwrap();
        let current = footprint
            .begin(minimum("21831px", "important", true))
            .unwrap();
        assert!(footprint.finish(old, true).is_none());
        assert_eq!(footprint.active.as_ref().unwrap().generation, current);
        let released = footprint.finish(current, true).unwrap();
        assert_eq!(released.original_minimum.value, "12rem");
        assert_eq!(released.original_minimum.priority, "important");
        assert!(released.original_minimum.attribute_present);
        assert!(footprint.finish(old, true).is_none());
        assert!(footprint.finish(current, true).is_none());
    }

    #[test]
    fn failed_or_shorter_replacement_and_cleanup_release_without_permanent_height() {
        for (loading, rendered, failed) in [(false, false, true), (false, true, false)] {
            let mut footprint = MountFootprint::default();
            let generation = footprint.begin(minimum("", "", true)).unwrap();
            let released = footprint
                .finish(generation, replacement_settled(loading, rendered, failed))
                .unwrap();
            assert!(released.original_minimum.value.is_empty());
            assert!(released.original_minimum.attribute_present);
            assert!(footprint.active.is_none());
        }
        let mut footprint = MountFootprint::default();
        let generation = footprint.begin(minimum("", "", false)).unwrap();
        let released = footprint.finish(generation, true).unwrap();
        assert!(!released.original_minimum.attribute_present);
        assert!(footprint.active.is_none());
    }

    #[test]
    fn absent_and_empty_inline_styles_preserve_unrelated_later_properties() {
        assert!(remove_empty_style_attribute(false, 0));
        assert!(!remove_empty_style_attribute(true, 0));
        assert!(!remove_empty_style_attribute(false, 1));
        assert!(!remove_empty_style_attribute(true, 1));
    }

    #[test]
    fn missing_outcome_and_invalid_measurement_do_not_license_release_or_guard() {
        assert!(!replacement_settled(false, false, false));
        assert!(!replacement_settled(true, true, true));
        for invalid in [-1.0, f64::NAN, f64::INFINITY] {
            assert_eq!(minimum_height(invalid), None);
        }
        assert_eq!(minimum_height(0.0), Some(0.0));
    }
}

use pos_core_kernel::prelude::*;

#[test]
fn duplicate_choice_selections_are_rejected_even_with_different_nested_selections() {
    let modifiers = pizza_modifiers();
    let toppings_id = component_id("01TOPPING");
    let pepperoni_id = component_id("01PEPPER");
    let placement_id = component_id("01PLACE");
    let left_id = component_id("01LEFT");
    let right_id = component_id("01RIGHT");

    let selections = Selections::new().with_prompt(
        toppings_id.clone(),
        vec![
            ChoiceSelection::new(pepperoni_id.clone(), 1).with_modifiers(
                Selections::new()
                    .with_prompt(placement_id.clone(), vec![ChoiceSelection::new(left_id, 1)]),
            ),
            ChoiceSelection::new(pepperoni_id.clone(), 1).with_modifiers(
                Selections::new().with_prompt(
                    placement_id.clone(),
                    vec![ChoiceSelection::new(right_id, 1)],
                ),
            ),
        ],
    );

    assert_eq!(
        modifiers.hydrate(&selections),
        Err(ModifierError::DuplicateSelection(pepperoni_id))
    );
}

#[test]
fn choice_rules_apply_to_selected_choice_quantity() {
    let modifiers = pizza_modifiers();
    let toppings_id = component_id("01TOPPING");
    let pepperoni_id = component_id("01PEPPER");
    let placement_id = component_id("01PLACE");

    let selections = Selections::new().with_prompt(
        toppings_id,
        vec![
            ChoiceSelection::new(pepperoni_id.clone(), 1).with_modifiers(
                Selections::new().with_prompt(
                    placement_id,
                    vec![ChoiceSelection::new(component_id("01LEFT"), 1)],
                ),
            ),
        ],
    );

    assert!(modifiers.hydrate(&selections).is_ok());

    let oversized_selection = Selections::new().with_prompt(
        component_id("01TOPPING"),
        vec![
            ChoiceSelection::new(pepperoni_id.clone(), 2).with_modifiers(
                Selections::new().with_prompt(
                    component_id("01PLACE"),
                    vec![ChoiceSelection::new(component_id("01WHOLE"), 1)],
                ),
            ),
        ],
    );

    assert_eq!(
        modifiers.hydrate(&oversized_selection),
        Err(ModifierError::ChoiceAboveMaximum {
            choice_id: pepperoni_id,
            max_select: 1,
            actual: 2
        })
    );
}

#[test]
fn configuration_dehydrates_nested_choices_and_round_trips() {
    let modifiers = pizza_modifiers();
    let toppings_id = component_id("01TOPPING");
    let pepperoni_id = component_id("01PEPPER");
    let mushroom_id = component_id("01MUSH");
    let placement_id = component_id("01PLACE");
    let left_id = component_id("01LEFT");
    let whole_id = component_id("01WHOLE");

    let selections = Selections::new().with_prompt(
        toppings_id,
        vec![
            ChoiceSelection::new(pepperoni_id.clone(), 1).with_modifiers(
                Selections::new()
                    .with_prompt(placement_id.clone(), vec![ChoiceSelection::new(left_id, 1)]),
            ),
            ChoiceSelection::new(mushroom_id, 1).with_modifiers(
                Selections::new()
                    .with_prompt(placement_id, vec![ChoiceSelection::new(whole_id, 1)]),
            ),
        ],
    );

    let configuration = modifiers.hydrate(&selections).unwrap();
    let dehydrated = configuration.dehydrate();

    assert_eq!(dehydrated, selections);
    assert_eq!(modifiers.hydrate(&dehydrated).unwrap(), configuration);
}

#[test]
fn same_prompt_and_choice_ids_can_be_reused_in_different_branches() {
    let modifiers = split_pizza_modifiers();
    let meats_id = component_id("01MEATS");
    let pepperoni_id = component_id("01PEPPER");
    let bacon_id = component_id("01BACON");
    let placement_id = component_id("01PLACE");
    let left_id = component_id("01LEFT");
    let right_id = component_id("01RIGHT");

    let selections = Selections::new().with_prompt(
        meats_id.clone(),
        vec![
            ChoiceSelection::new(pepperoni_id, 1).with_modifiers(
                Selections::new()
                    .with_prompt(placement_id.clone(), vec![ChoiceSelection::new(left_id, 1)]),
            ),
            ChoiceSelection::new(bacon_id, 1).with_modifiers(Selections::new().with_prompt(
                placement_id.clone(),
                vec![ChoiceSelection::new(right_id, 1)],
            )),
        ],
    );

    let configuration = modifiers.hydrate(&selections).unwrap();
    let meats = configuration.prompt(&meats_id).unwrap();

    assert_eq!(
        meats.choices()[0]
            .modifiers()
            .unwrap()
            .prompt(&placement_id)
            .unwrap()
            .choices()[0]
            .choice_id(),
        &component_id("01LEFT")
    );
    assert_eq!(
        meats.choices()[1]
            .modifiers()
            .unwrap()
            .prompt(&placement_id)
            .unwrap()
            .choices()[0]
            .choice_id(),
        &component_id("01RIGHT")
    );
}

#[test]
fn same_modifiers_can_hold_repeated_prompt_ids_as_ordered_instances() {
    let sauce_id = component_id("01SAUCE");
    let hot_id = component_id("01HOT");
    let ranch_id = component_id("01RANCH");
    let modifiers = Modifiers::new(vec![
        Prompt::new(
            sauce_id.clone(),
            "Wing sauce",
            None,
            vec![Rule::Min(1), Rule::Max(1)],
            Vec::new(),
            vec![
                Choice::new(component_id("01MILD"), "Mild", Vec::new(), Vec::new()).unwrap(),
                Choice::new(hot_id.clone(), "Hot", Vec::new(), Vec::new()).unwrap(),
            ],
        )
        .unwrap(),
        Prompt::new(
            sauce_id.clone(),
            "Dipping sauce",
            None,
            vec![Rule::Min(1), Rule::Max(1)],
            Vec::new(),
            vec![
                Choice::new(ranch_id.clone(), "Ranch", Vec::new(), Vec::new()).unwrap(),
                Choice::new(
                    component_id("01BLUE"),
                    "Blue cheese",
                    Vec::new(),
                    Vec::new(),
                )
                .unwrap(),
            ],
        )
        .unwrap(),
    ]);
    let selections = Selections::new()
        .with_prompt_instance(PromptSelection::new(
            sauce_id.clone(),
            vec![ChoiceSelection::new(hot_id, 1)],
        ))
        .with_prompt_instance(PromptSelection::new(
            sauce_id.clone(),
            vec![ChoiceSelection::new(ranch_id, 1)],
        ));

    let configuration = modifiers.hydrate(&selections).unwrap();
    let dehydrated = configuration.dehydrate();

    assert_eq!(configuration.prompts().len(), 2);
    assert_eq!(configuration.prompts()[0].prompt_id(), &sauce_id);
    assert_eq!(configuration.prompts()[1].prompt_id(), &sauce_id);
    assert_eq!(
        configuration.prompt_at(&sauce_id, 1).unwrap().choices()[0].choice_id(),
        &component_id("01RANCH")
    );
    assert_eq!(dehydrated, selections);
    assert_eq!(modifiers.hydrate(&dehydrated).unwrap(), configuration);
}

#[test]
fn three_level_nested_selection_dehydrates_and_round_trips() {
    let modifiers = deeply_nested_modifiers();
    let toppings_id = component_id("01TOPPING");
    let pepperoni_id = component_id("01PEPPER");
    let placement_id = component_id("01PLACE");
    let left_id = component_id("01LEFT");
    let amount_id = component_id("01AMOUNT");
    let extra_id = component_id("01EXTRA");

    let selections = Selections::new().with_prompt(
        toppings_id,
        vec![
            ChoiceSelection::new(pepperoni_id, 1).with_modifiers(Selections::new().with_prompt(
                placement_id,
                vec![ChoiceSelection::new(left_id, 1).with_modifiers(
                    Selections::new()
                        .with_prompt(amount_id, vec![ChoiceSelection::new(extra_id, 1)]),
                )],
            )),
        ],
    );

    let configuration = modifiers.hydrate(&selections).unwrap();
    let dehydrated = configuration.dehydrate();

    assert_eq!(dehydrated, selections);
    assert_eq!(modifiers.hydrate(&dehydrated).unwrap(), configuration);
}

#[test]
fn defaults_dehydrate_into_the_effective_selection_snapshot() {
    let cheese_id = component_id("01CHEESE");
    let american_id = component_id("01AMERICAN");
    let modifiers = Modifiers::new(vec![
        Prompt::new(
            cheese_id.clone(),
            "Cheese",
            None,
            vec![Rule::Min(1), Rule::Max(1)],
            Vec::new(),
            vec![
                Choice::new(
                    american_id.clone(),
                    "American",
                    vec![Rule::Default(1)],
                    Vec::new(),
                )
                .unwrap(),
                Choice::new(component_id("01SWISS"), "Swiss", Vec::new(), Vec::new()).unwrap(),
            ],
        )
        .unwrap(),
    ]);

    let configuration = modifiers.hydrate(&Selections::new()).unwrap();
    let dehydrated = configuration.dehydrate();
    let expected = Selections::new().with_prompt(
        cheese_id.clone(),
        vec![ChoiceSelection::new(american_id, 1).with_source(SelectionSource::Default)],
    );

    assert_eq!(dehydrated, expected);
    assert_eq!(configuration.prompt(&cheese_id).unwrap().choices().len(), 1);
    assert_eq!(modifiers.hydrate(&dehydrated).unwrap(), configuration);
}

#[test]
fn snapshot_preserves_default_selections_labels_and_price_facts() {
    let cheese_id = component_id("01CHEESE");
    let american_id = component_id("01AMERICAN");
    let modifiers = Modifiers::new(vec![
        Prompt::new(
            cheese_id.clone(),
            "Cheese",
            Some("Choose the included cheese".to_owned()),
            vec![Rule::Min(1), Rule::Max(1)],
            Vec::new(),
            vec![
                Choice::new(
                    american_id.clone(),
                    "American",
                    vec![Rule::Default(1)],
                    Vec::new(),
                )
                .unwrap()
                .with_price(ChoicePrice::flat_amount(usd(75)).unwrap()),
                Choice::new(component_id("01SWISS"), "Swiss", Vec::new(), Vec::new()).unwrap(),
            ],
        )
        .unwrap(),
    ]);

    let configuration = modifiers.hydrate(&Selections::new()).unwrap();
    let snapshot = configuration
        .snapshot(
            &usd(1000),
            ModifierPricingPolicy::new(false, RoundingStrategy::CentRoundUp),
        )
        .unwrap();
    let prompt = snapshot.prompt(&cheese_id).unwrap();
    let choice = &prompt.choices()[0];
    let contribution = &snapshot.price().contributions()[0];

    assert_eq!(prompt.title(), "Cheese");
    assert_eq!(prompt.label().label_id(), Some(&label_id("01CHEESE-TITLE")));
    assert_eq!(prompt.description(), Some("Choose the included cheese"));
    assert_eq!(
        prompt.description_label().unwrap().label_id(),
        Some(&label_id("01CHEESE-DESCRIPTION"))
    );
    assert_eq!(choice.choice_id(), &american_id);
    assert_eq!(choice.title(), "American");
    assert_eq!(
        choice.label().label_id(),
        Some(&label_id("01AMERICAN-TITLE"))
    );
    assert_eq!(choice.quantity(), 1);
    assert_eq!(choice.source(), SelectionSource::Default);
    assert_eq!(
        choice
            .price_definition()
            .flat_amount_ref()
            .unwrap()
            .amount_minor(),
        75
    );
    assert_eq!(contribution.choice_id(), &american_id);
    assert_eq!(contribution.source(), SelectionSource::Default);
    assert_eq!(contribution.amount().amount_minor(), 75);
    assert_eq!(snapshot.price().total().amount_minor(), 75);
}

#[test]
fn modifier_hydration_resolves_labels_for_consumer_profiles() {
    let prep = consumer_attribute_id("PREP");
    let profile = ConsumerProfile::new([prep.clone()]).unwrap();
    let sauce_id = component_id("01SAUCE");
    let ranch_id = component_id("01RANCH");
    let prompt_label = Label::new(label_id("SAUCE-PROMPT"), "Sauce")
        .unwrap()
        .with_value(ConsumerProfile::new([prep.clone()]).unwrap(), "SAUCE")
        .unwrap();
    let choice_label = Label::new(label_id("RANCH-CHOICE"), "Ranch")
        .unwrap()
        .with_value(ConsumerProfile::new([prep]).unwrap(), "RCH")
        .unwrap();
    let modifiers = Modifiers::new(vec![
        Prompt::new_labeled(
            sauce_id.clone(),
            prompt_label,
            None,
            vec![Rule::Min(1), Rule::Max(1)],
            Vec::new(),
            vec![
                Choice::new_labeled(ranch_id.clone(), choice_label, Vec::new(), Vec::new())
                    .unwrap(),
            ],
        )
        .unwrap(),
    ]);
    let selections =
        Selections::new().with_prompt(sauce_id.clone(), vec![ChoiceSelection::new(ranch_id, 1)]);

    let configuration = modifiers
        .hydrate_for_profile(&selections, &profile)
        .unwrap();
    let prompt = configuration.prompt(&sauce_id).unwrap();
    let choice = &prompt.choices()[0];

    assert_eq!(prompt.title(), "SAUCE");
    assert_eq!(prompt.label().label_id(), Some(&label_id("SAUCE-PROMPT")));
    assert_eq!(choice.title(), "RCH");
    assert_eq!(choice.label().label_id(), Some(&label_id("RANCH-CHOICE")));
}

#[test]
fn choice_media_is_definition_metadata_and_not_configuration_state() {
    let sauce_id = component_id("01SAUCE");
    let ranch_id = component_id("01RANCH");
    let media = MediaCollection::new(vec![
        Media::new(media_id("RANCH-PHOTO"), mime("image/webp"))
            .with_dimensions(MediaDimensions::new(800, 600).unwrap()),
    ])
    .unwrap();
    let choice = Choice::new(ranch_id.clone(), "Ranch", Vec::new(), Vec::new())
        .unwrap()
        .with_media(media.clone());

    assert_eq!(choice.media(), &media);

    let modifiers = Modifiers::new(vec![
        Prompt::new(
            sauce_id.clone(),
            "Sauce",
            None,
            vec![Rule::Min(1), Rule::Max(1)],
            Vec::new(),
            vec![choice],
        )
        .unwrap(),
    ]);
    let selections =
        Selections::new().with_prompt(sauce_id.clone(), vec![ChoiceSelection::new(ranch_id, 1)]);
    let configuration = modifiers.hydrate(&selections).unwrap();
    let snapshot = configuration
        .snapshot(&usd(0), ModifierPricingPolicy::default())
        .unwrap();

    assert_eq!(configuration.prompt(&sauce_id).unwrap().choices().len(), 1);
    assert_eq!(snapshot.prompt(&sauce_id).unwrap().choices().len(), 1);
}

#[test]
fn defaults_must_satisfy_prompt_min_and_max_rules() {
    let below_min = Modifiers::new(vec![
        Prompt::new(
            component_id("01CHEESE"),
            "Cheese",
            None,
            vec![Rule::Min(2), Rule::Max(2)],
            Vec::new(),
            vec![
                Choice::new(
                    component_id("01AMERICAN"),
                    "American",
                    vec![Rule::Default(1)],
                    Vec::new(),
                )
                .unwrap(),
                Choice::new(component_id("01SWISS"), "Swiss", Vec::new(), Vec::new()).unwrap(),
            ],
        )
        .unwrap(),
    ]);

    assert_eq!(
        below_min.hydrate(&Selections::new()),
        Err(ModifierError::BelowMinimum {
            min_select: 2,
            actual: 1
        })
    );

    let above_max = Modifiers::new(vec![
        Prompt::new(
            component_id("01CHEESE"),
            "Cheese",
            None,
            vec![Rule::Max(1)],
            Vec::new(),
            vec![
                Choice::new(
                    component_id("01AMERICAN"),
                    "American",
                    vec![Rule::Default(1)],
                    Vec::new(),
                )
                .unwrap(),
                Choice::new(
                    component_id("01SWISS"),
                    "Swiss",
                    vec![Rule::Default(1)],
                    Vec::new(),
                )
                .unwrap(),
            ],
        )
        .unwrap(),
    ]);

    assert_eq!(
        above_max.hydrate(&Selections::new()),
        Err(ModifierError::AboveMaximum {
            max_select: 1,
            actual: 2
        })
    );
}

#[test]
fn duplicate_min_or_max_rules_are_rejected() {
    assert_eq!(
        Prompt::new(
            component_id("01CHEESE"),
            "Cheese",
            None,
            vec![Rule::Min(1), Rule::Min(2)],
            Vec::new(),
            vec![Choice::new(component_id("01SWISS"), "Swiss", Vec::new(), Vec::new()).unwrap()],
        ),
        Err(ModifierError::DuplicateRule(RuleKind::Min))
    );

    assert_eq!(
        Choice::new(
            component_id("01BACON"),
            "Bacon",
            vec![Rule::Max(1), Rule::Max(2)],
            Vec::new(),
        ),
        Err(ModifierError::DuplicateRule(RuleKind::Max))
    );
}

#[test]
fn definitions_reject_empty_titles_invalid_prompt_rules_and_empty_required_prompts() {
    assert_eq!(
        Prompt::new(
            component_id("01EMPTY"),
            "   ",
            None,
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
        Err(ModifierError::EmptyPromptTitle)
    );

    assert_eq!(
        Choice::new(component_id("01EMPTY"), "", Vec::new(), Vec::new()),
        Err(ModifierError::EmptyChoiceTitle)
    );

    assert_eq!(
        Prompt::new(
            component_id("01CHEESE"),
            "Cheese",
            None,
            vec![Rule::Default(1)],
            Vec::new(),
            vec![Choice::new(component_id("01SWISS"), "Swiss", Vec::new(), Vec::new()).unwrap()],
        ),
        Err(ModifierError::DefaultRuleOnPrompt)
    );

    assert_eq!(
        Prompt::new(
            component_id("01SAUCE"),
            "Sauce",
            None,
            vec![Rule::Min(1)],
            Vec::new(),
            Vec::new(),
        ),
        Err(ModifierError::RequiredPromptHasNoChoices {
            prompt_id: component_id("01SAUCE"),
            min_select: 1
        })
    );
}

#[test]
fn definitions_reject_invalid_min_max_constraints_and_duplicate_choice_definitions() {
    assert_eq!(
        Prompt::new(
            component_id("01CHEESE"),
            "Cheese",
            None,
            vec![Rule::Min(2), Rule::Max(1)],
            Vec::new(),
            vec![Choice::new(component_id("01SWISS"), "Swiss", Vec::new(), Vec::new()).unwrap()],
        ),
        Err(ModifierError::InvalidConstraints {
            min_select: 2,
            max_select: 1
        })
    );

    assert_eq!(
        Choice::new(
            component_id("01BACON"),
            "Bacon",
            vec![Rule::Min(3), Rule::Max(2)],
            Vec::new(),
        ),
        Err(ModifierError::InvalidConstraints {
            min_select: 3,
            max_select: 2
        })
    );

    assert_eq!(
        Prompt::new(
            component_id("01CHEESE"),
            "Cheese",
            None,
            Vec::new(),
            Vec::new(),
            vec![
                Choice::new(component_id("01SWISS"), "Swiss", Vec::new(), Vec::new()).unwrap(),
                Choice::new(
                    component_id("01SWISS"),
                    "Swiss again",
                    Vec::new(),
                    Vec::new()
                )
                .unwrap(),
            ],
        ),
        Err(ModifierError::DuplicateChoice(component_id("01SWISS")))
    );
}

#[test]
fn choice_default_rules_are_unique_nonzero_and_within_choice_bounds() {
    let bacon_id = component_id("01BACON");

    assert_eq!(
        Choice::new(
            bacon_id.clone(),
            "Bacon",
            vec![Rule::Default(1), Rule::Default(2)],
            Vec::new(),
        ),
        Err(ModifierError::DuplicateDefault(bacon_id.clone()))
    );

    assert_eq!(
        Choice::new(
            bacon_id.clone(),
            "Bacon",
            vec![Rule::Default(0)],
            Vec::new(),
        ),
        Err(ModifierError::ZeroDefaultQuantity(bacon_id.clone()))
    );

    assert_eq!(
        Choice::new(
            bacon_id.clone(),
            "Bacon",
            vec![Rule::Min(2), Rule::Default(1)],
            Vec::new(),
        ),
        Err(ModifierError::ChoiceBelowMinimum {
            choice_id: bacon_id.clone(),
            min_select: 2,
            actual: 1
        })
    );

    assert_eq!(
        Choice::new(
            bacon_id.clone(),
            "Bacon",
            vec![Rule::Max(1), Rule::Default(2)],
            Vec::new(),
        ),
        Err(ModifierError::ChoiceAboveMaximum {
            choice_id: bacon_id,
            max_select: 1,
            actual: 2
        })
    );
}

#[test]
fn prompt_min_and_max_rules_validate_selection_counts() {
    let cheddar_id = component_id("01CHEDDAR");
    let swiss_id = component_id("01SWISS");
    let prompt = Prompt::new(
        component_id("01CHEESE"),
        "Cheese",
        None,
        vec![Rule::Min(1), Rule::Max(2)],
        Vec::new(),
        vec![
            Choice::new(cheddar_id.clone(), "Cheddar", Vec::new(), Vec::new()).unwrap(),
            Choice::new(swiss_id.clone(), "Swiss", Vec::new(), Vec::new()).unwrap(),
            Choice::new(
                component_id("01AMERICAN"),
                "American",
                Vec::new(),
                Vec::new(),
            )
            .unwrap(),
        ],
    )
    .unwrap();

    assert_eq!(
        prompt.validate_selections(&[]),
        Err(ModifierError::BelowMinimum {
            min_select: 1,
            actual: 0
        })
    );
    assert!(
        prompt
            .validate_selections(&[ChoiceSelection::new(cheddar_id, 1)])
            .is_ok()
    );
    assert!(
        prompt
            .validate_selections(&[
                ChoiceSelection::new(swiss_id, 1),
                ChoiceSelection::new(component_id("01AMERICAN"), 1),
            ])
            .is_ok()
    );
    assert_eq!(
        prompt.validate_selections(&[
            ChoiceSelection::new(component_id("01CHEDDAR"), 1),
            ChoiceSelection::new(component_id("01SWISS"), 1),
            ChoiceSelection::new(component_id("01AMERICAN"), 1),
        ]),
        Err(ModifierError::AboveMaximum {
            max_select: 2,
            actual: 3
        })
    );
}

#[test]
fn prompt_rejects_zero_duplicate_and_unknown_choice_selections() {
    let ranch_id = component_id("01RANCH");
    let prompt = Prompt::new(
        component_id("01SAUCE"),
        "Sauce",
        None,
        vec![Rule::Max(2)],
        Vec::new(),
        vec![Choice::new(ranch_id.clone(), "Ranch", Vec::new(), Vec::new()).unwrap()],
    )
    .unwrap();

    assert_eq!(
        prompt.validate_selections(&[ChoiceSelection::new(ranch_id.clone(), 0)]),
        Err(ModifierError::ZeroQuantity(ranch_id.clone()))
    );
    assert_eq!(
        prompt.validate_selections(&[
            ChoiceSelection::new(ranch_id.clone(), 1),
            ChoiceSelection::new(ranch_id.clone(), 1),
        ]),
        Err(ModifierError::DuplicateSelection(ranch_id.clone()))
    );
    assert_eq!(
        prompt.validate_selections(&[ChoiceSelection::new(component_id("01HOT"), 1)]),
        Err(ModifierError::UnknownSelection(component_id("01HOT")))
    );
}

#[test]
fn prompt_rejects_selection_count_overflow() {
    let first_id = component_id("01FIRST");
    let second_id = component_id("01SECOND");
    let prompt = Prompt::new(
        component_id("01COUNT"),
        "Count",
        None,
        Vec::new(),
        Vec::new(),
        vec![
            Choice::new(first_id.clone(), "First", Vec::new(), Vec::new()).unwrap(),
            Choice::new(second_id.clone(), "Second", Vec::new(), Vec::new()).unwrap(),
        ],
    )
    .unwrap();

    assert_eq!(
        prompt.validate_selections(&[
            ChoiceSelection::new(first_id, u32::MAX),
            ChoiceSelection::new(second_id, 1),
        ]),
        Err(ModifierError::SelectionCountOverflow)
    );
}

#[test]
fn max_zero_prompt_allows_no_selection_and_rejects_any_selection() {
    let prompt_id = component_id("01LOCKED");
    let choice_id = component_id("01CHOICE");
    let modifiers = Modifiers::new(vec![
        Prompt::new(
            prompt_id.clone(),
            "Locked",
            None,
            vec![Rule::Max(0)],
            Vec::new(),
            vec![Choice::new(choice_id.clone(), "Choice", Vec::new(), Vec::new()).unwrap()],
        )
        .unwrap(),
    ]);

    assert!(modifiers.hydrate(&Selections::new()).is_ok());
    assert_eq!(
        modifiers.hydrate(
            &Selections::new().with_prompt(prompt_id, vec![ChoiceSelection::new(choice_id, 1)])
        ),
        Err(ModifierError::AboveMaximum {
            max_select: 0,
            actual: 1
        })
    );
}

#[test]
fn explicit_selections_replace_defaults_for_a_prompt() {
    let american_id = component_id("01AMERICAN");
    let swiss_id = component_id("01SWISS");
    let modifiers = Modifiers::new(vec![
        Prompt::new(
            component_id("01CHEESE"),
            "Cheese",
            None,
            vec![Rule::Min(1), Rule::Max(1)],
            Vec::new(),
            vec![
                Choice::new(
                    american_id.clone(),
                    "American",
                    vec![Rule::Default(1)],
                    Vec::new(),
                )
                .unwrap(),
                Choice::new(swiss_id.clone(), "Swiss", Vec::new(), Vec::new()).unwrap(),
            ],
        )
        .unwrap(),
    ]);
    let selections = Selections::new().with_prompt(
        component_id("01CHEESE"),
        vec![ChoiceSelection::new(swiss_id.clone(), 1)],
    );

    let configuration = modifiers.hydrate(&selections).unwrap();
    let choices = configuration
        .prompt(&component_id("01CHEESE"))
        .unwrap()
        .choices();

    assert_eq!(choices.len(), 1);
    assert_eq!(choices[0].choice_id(), &swiss_id);
    assert_eq!(choices[0].source(), SelectionSource::Explicit);
    assert_ne!(choices[0].choice_id(), &american_id);
}

#[test]
fn nested_required_prompts_must_be_satisfied_when_parent_choice_is_selected() {
    let modifiers = pizza_modifiers();
    let selections = Selections::new().with_prompt(
        component_id("01TOPPING"),
        vec![ChoiceSelection::new(component_id("01PEPPER"), 1)],
    );

    assert_eq!(
        modifiers.hydrate(&selections),
        Err(ModifierError::BelowMinimum {
            min_select: 1,
            actual: 0
        })
    );
}

#[test]
fn nested_defaults_dehydrate_into_the_effective_selection_snapshot() {
    let parent_id = component_id("01PARENT");
    let child_id = component_id("01CHILD");
    let default_child_id = component_id("01DEFAULT");
    let modifiers = Modifiers::new(vec![
        Prompt::new(
            parent_id.clone(),
            "Parent",
            None,
            vec![Rule::Min(1), Rule::Max(1)],
            Vec::new(),
            vec![
                Choice::new(component_id("01CHOICE"), "Choice", Vec::new(), Vec::new())
                    .unwrap()
                    .with_modifiers(Modifiers::new(vec![
                        Prompt::new(
                            child_id.clone(),
                            "Child",
                            None,
                            vec![Rule::Min(1), Rule::Max(1)],
                            Vec::new(),
                            vec![
                                Choice::new(
                                    default_child_id.clone(),
                                    "Default child",
                                    vec![Rule::Default(1)],
                                    Vec::new(),
                                )
                                .unwrap(),
                                Choice::new(
                                    component_id("01OTHER"),
                                    "Other",
                                    Vec::new(),
                                    Vec::new(),
                                )
                                .unwrap(),
                            ],
                        )
                        .unwrap(),
                    ])),
            ],
        )
        .unwrap(),
    ]);
    let selections = Selections::new().with_prompt(
        parent_id.clone(),
        vec![ChoiceSelection::new(component_id("01CHOICE"), 1)],
    );

    let configuration = modifiers.hydrate(&selections).unwrap();
    let nested = configuration.prompt(&parent_id).unwrap().choices()[0]
        .modifiers()
        .unwrap()
        .prompt(&child_id)
        .unwrap();

    assert_eq!(nested.choices()[0].choice_id(), &default_child_id);
    assert_eq!(nested.choices()[0].source(), SelectionSource::Default);
    assert_eq!(
        configuration.dehydrate(),
        Selections::new().with_prompt(
            parent_id.clone(),
            vec![
                ChoiceSelection::new(component_id("01CHOICE"), 1).with_modifiers(
                    Selections::new().with_prompt(
                        child_id.clone(),
                        vec![
                            ChoiceSelection::new(default_child_id.clone(), 1)
                                .with_source(SelectionSource::Default),
                        ],
                    ),
                )
            ],
        )
    );
    assert_eq!(
        modifiers.hydrate(&configuration.dehydrate()).unwrap(),
        configuration
    );
}

#[test]
fn strict_hydrate_rejects_unknown_prompt_selections_and_extra_prompt_occurrences() {
    let sauce_id = component_id("01SAUCE");
    let ranch_id = component_id("01RANCH");
    let modifiers = Modifiers::new(vec![
        Prompt::new(
            sauce_id.clone(),
            "Sauce",
            None,
            vec![Rule::Max(1)],
            Vec::new(),
            vec![Choice::new(ranch_id.clone(), "Ranch", Vec::new(), Vec::new()).unwrap()],
        )
        .unwrap(),
    ]);

    assert_eq!(
        modifiers.hydrate(&Selections::new().with_prompt(
            component_id("01UNKNOWN"),
            vec![ChoiceSelection::new(ranch_id.clone(), 1)]
        )),
        Err(ModifierError::UnknownPromptSelection(component_id(
            "01UNKNOWN"
        )))
    );

    let extra_occurrence = Selections::new()
        .with_prompt_instance(PromptSelection::new(
            sauce_id.clone(),
            vec![ChoiceSelection::new(ranch_id.clone(), 1)],
        ))
        .with_prompt_instance(PromptSelection::new(
            sauce_id.clone(),
            vec![ChoiceSelection::new(ranch_id, 1)],
        ));

    assert_eq!(
        modifiers.hydrate(&extra_occurrence),
        Err(ModifierError::UnknownPromptSelection(sauce_id))
    );
}

#[test]
fn strict_hydrate_rejects_nested_selections_for_terminal_choices() {
    let modifiers = Modifiers::new(vec![
        Prompt::new(
            component_id("01SAUCE"),
            "Sauce",
            None,
            vec![Rule::Max(1)],
            Vec::new(),
            vec![Choice::new(component_id("01RANCH"), "Ranch", Vec::new(), Vec::new()).unwrap()],
        )
        .unwrap(),
    ]);
    let selections = Selections::new().with_prompt(
        component_id("01SAUCE"),
        vec![
            ChoiceSelection::new(component_id("01RANCH"), 1).with_modifiers(
                Selections::new().with_prompt(
                    component_id("01GHOST"),
                    vec![ChoiceSelection::new(component_id("01NOPE"), 1)],
                ),
            ),
        ],
    );

    assert_eq!(
        modifiers.hydrate(&selections),
        Err(ModifierError::UnexpectedNestedSelections(component_id(
            "01RANCH"
        )))
    );
}

#[test]
fn optional_prompt_without_selection_hydrates_empty_configuration() {
    let prompt_id = component_id("01SAUCE");
    let modifiers = Modifiers::new(vec![
        Prompt::new(
            prompt_id.clone(),
            "Sauce",
            None,
            vec![Rule::Max(1)],
            Vec::new(),
            vec![Choice::new(component_id("01RANCH"), "Ranch", Vec::new(), Vec::new()).unwrap()],
        )
        .unwrap(),
    ]);

    let configuration = modifiers.hydrate(&Selections::new()).unwrap();

    assert_eq!(configuration.prompt(&prompt_id).unwrap().choices(), &[]);
    assert!(configuration.dehydrate().is_empty());
}

#[test]
fn scheduled_choice_requires_evaluation_time_when_selected() {
    let modifiers = scheduled_choice_modifiers(false);
    let brunch_id = component_id("01BRUNCH");
    let selections = Selections::new().with_prompt(
        component_id("01SPECIAL"),
        vec![ChoiceSelection::new(brunch_id.clone(), 1)],
    );

    assert_eq!(
        modifiers.hydrate(&selections),
        Err(ModifierError::ScheduledChoiceRequiresEvaluationTime(
            brunch_id
        ))
    );
}

#[test]
fn scheduled_choice_hydrates_only_inside_its_schedule() {
    let modifiers = scheduled_choice_modifiers(false);
    let brunch_id = component_id("01BRUNCH");
    let selections = Selections::new().with_prompt(
        component_id("01SPECIAL"),
        vec![ChoiceSelection::new(brunch_id.clone(), 1)],
    );

    assert!(
        modifiers
            .hydrate_at(&selections, &evaluation_time(100, 2026, 5, 22, 12, 0, 0))
            .is_ok()
    );
    assert_eq!(
        modifiers.hydrate_at(&selections, &evaluation_time(100, 2026, 5, 22, 15, 0, 0),),
        Err(ModifierError::UnavailableChoiceSelection(brunch_id))
    );
}

#[test]
fn scheduled_choice_is_not_visible_outside_its_schedule() {
    let modifiers = scheduled_choice_modifiers(false);
    let prompt = &modifiers.prompts()[0];
    let inside = evaluation_time(100, 2026, 5, 22, 12, 0, 0);
    let outside = evaluation_time(100, 2026, 5, 22, 15, 0, 0);

    assert_eq!(prompt.visible_choices_at(&inside).len(), 1);
    assert!(prompt.visible_choices_at(&outside).is_empty());

    let mut visible_nodes = Vec::new();
    modifiers.walk_visible_at(&outside, &mut |node| match node {
        ModifierNode::Modifiers(_) => visible_nodes.push("modifiers".to_owned()),
        ModifierNode::Prompt(prompt) => visible_nodes.push(format!("prompt:{}", prompt.title())),
        ModifierNode::Choice(choice) => visible_nodes.push(format!("choice:{}", choice.title())),
    });

    assert_eq!(visible_nodes, vec!["modifiers", "prompt:Specials"]);
}

#[test]
fn scheduled_default_choice_uses_supplied_evaluation_time() {
    let modifiers = scheduled_choice_modifiers(true);

    assert_eq!(
        modifiers.hydrate(&Selections::new()),
        Err(ModifierError::ScheduledChoiceRequiresEvaluationTime(
            component_id("01BRUNCH")
        ))
    );

    let inside = modifiers
        .hydrate_at(
            &Selections::new(),
            &evaluation_time(100, 2026, 5, 22, 12, 0, 0),
        )
        .unwrap();

    assert_eq!(
        inside.prompt(&component_id("01SPECIAL")).unwrap().choices()[0].choice_id(),
        &component_id("01BRUNCH")
    );
    assert_eq!(
        modifiers.hydrate_at(
            &Selections::new(),
            &evaluation_time(100, 2026, 5, 22, 15, 0, 0),
        ),
        Err(ModifierError::BelowMinimum {
            min_select: 1,
            actual: 0
        })
    );
}

#[test]
fn flat_modifier_price_is_multiplied_by_selected_factor() {
    let modifiers = pizza_modifiers();
    let configuration = modifiers
        .hydrate(&Selections::new().with_prompt(
            component_id("01TOPPING"),
            vec![
                ChoiceSelection::new(component_id("01PEPPER"), 1).with_modifiers(
                    Selections::new().with_prompt(
                        component_id("01PLACE"),
                        vec![ChoiceSelection::new(component_id("01LEFT"), 1)],
                    ),
                ),
            ],
        ))
        .unwrap();

    let priced = configuration
        .price(&usd(1800), ModifierPricingPolicy::default())
        .unwrap();

    assert_eq!(priced.total().amount_minor(), 100);
    assert_eq!(priced.contributions().len(), 1);
    assert_eq!(
        priced.contributions()[0].choice_id(),
        &component_id("01PEPPER")
    );
    assert_eq!(
        priced.contributions()[0].factors()[0].choice_id(),
        &component_id("01LEFT")
    );
}

#[test]
fn invariant_rate_price_adds_to_flat_amount_without_floats() {
    let modifiers = Modifiers::new(vec![
        Prompt::new(
            component_id("01UPGRADE"),
            "Upgrade",
            None,
            Vec::new(),
            Vec::new(),
            vec![
                Choice::new(
                    component_id("01PREMIUM"),
                    "Premium topping",
                    Vec::new(),
                    Vec::new(),
                )
                .unwrap()
                .with_price(
                    ChoicePrice::flat_amount(usd(100))
                        .unwrap()
                        .with_invariant_rate(Rate::percent(10)),
                ),
            ],
        )
        .unwrap(),
    ]);
    let configuration = modifiers
        .hydrate(&Selections::new().with_prompt(
            component_id("01UPGRADE"),
            vec![ChoiceSelection::new(component_id("01PREMIUM"), 1)],
        ))
        .unwrap();

    let priced = configuration
        .price(&usd(1800), ModifierPricingPolicy::default())
        .unwrap();

    assert_eq!(priced.total().amount_minor(), 280);
}

#[test]
fn nested_factors_multiply_up_to_the_nearest_priced_ancestor() {
    let configuration = deeply_nested_priced_modifiers()
        .hydrate(&Selections::new().with_prompt(
            component_id("01TOPPING"),
            vec![
                ChoiceSelection::new(component_id("01PEPPER"), 1).with_modifiers(
                    Selections::new().with_prompt(
                        component_id("01PLACE"),
                        vec![
                            ChoiceSelection::new(component_id("01LEFT"), 1).with_modifiers(
                                Selections::new().with_prompt(
                                    component_id("01AMOUNT"),
                                    vec![ChoiceSelection::new(component_id("01HALF"), 1)],
                                ),
                            ),
                        ],
                    ),
                ),
            ],
        ))
        .unwrap();

    let priced = configuration
        .price(&usd(1800), ModifierPricingPolicy::default())
        .unwrap();

    assert_eq!(priced.total().amount_minor(), 50);
    assert_eq!(priced.contributions()[0].factors().len(), 2);
}

#[test]
fn defaults_are_free_unless_pricing_policy_says_otherwise() {
    let modifiers = Modifiers::new(vec![
        Prompt::new(
            component_id("01TOPPING"),
            "Toppings",
            None,
            vec![Rule::Min(1), Rule::Max(1)],
            Vec::new(),
            vec![
                Choice::new(
                    component_id("01PEPPER"),
                    "Pepperoni",
                    vec![Rule::Default(1)],
                    Vec::new(),
                )
                .unwrap()
                .with_price(ChoicePrice::flat_amount(usd(200)).unwrap()),
            ],
        )
        .unwrap(),
    ]);
    let configuration = modifiers.hydrate(&Selections::new()).unwrap();

    let default_free = configuration
        .price(&usd(1800), ModifierPricingPolicy::default())
        .unwrap();
    let default_charged = configuration
        .price(
            &usd(1800),
            ModifierPricingPolicy::new(false, RoundingStrategy::CentRoundUp),
        )
        .unwrap();

    assert_eq!(default_free.total().amount_minor(), 0);
    assert_eq!(default_free.contributions(), &[]);
    assert_eq!(default_charged.total().amount_minor(), 200);
}

#[test]
fn unconsumed_root_factor_is_invalid() {
    let modifiers = Modifiers::new(vec![
        Prompt::new(
            component_id("01PLACE"),
            "Placement",
            None,
            vec![Rule::Min(1), Rule::Max(1)],
            Vec::new(),
            vec![
                Choice::new(component_id("01LEFT"), "Left", Vec::new(), Vec::new())
                    .unwrap()
                    .with_price(ChoicePrice::from_factor(Rate::percent(50))),
            ],
        )
        .unwrap(),
    ]);
    let configuration = modifiers
        .hydrate(&Selections::new().with_prompt(
            component_id("01PLACE"),
            vec![ChoiceSelection::new(component_id("01LEFT"), 1)],
        ))
        .unwrap();

    assert_eq!(
        configuration.price(&usd(1800), ModifierPricingPolicy::default()),
        Err(ModifierError::UnconsumedPriceFactor(component_id("01LEFT")))
    );
}

#[test]
fn titled_modifiers_expose_title_without_affecting_hydration() {
    let prompt_id = component_id("01SAUCE");
    let modifiers = Modifiers::titled(
        "Sauces",
        vec![
            Prompt::new(
                prompt_id.clone(),
                "Sauce",
                None,
                vec![Rule::Max(1)],
                Vec::new(),
                vec![
                    Choice::new(component_id("01RANCH"), "Ranch", Vec::new(), Vec::new()).unwrap(),
                ],
            )
            .unwrap(),
        ],
    );

    assert_eq!(modifiers.title(), Some("Sauces"));
    assert!(modifiers.hydrate(&Selections::new()).is_ok());
    assert_eq!(modifiers.prompts()[0].prompt_id(), &prompt_id);
}

fn pizza_modifiers() -> Modifiers {
    let placement = placement_modifiers();
    let toppings = Prompt::new(
        component_id("01TOPPING"),
        "Toppings",
        None,
        vec![Rule::Max(5)],
        Vec::new(),
        vec![
            Choice::new(
                component_id("01PEPPER"),
                "Pepperoni",
                vec![Rule::Max(1)],
                Vec::new(),
            )
            .unwrap()
            .with_price(ChoicePrice::flat_amount(usd(200)).unwrap())
            .with_modifiers(placement.clone()),
            Choice::new(component_id("01MUSH"), "Mushrooms", Vec::new(), Vec::new())
                .unwrap()
                .with_price(ChoicePrice::flat_amount(usd(150)).unwrap())
                .with_modifiers(placement),
        ],
    )
    .unwrap();

    Modifiers::new(vec![toppings])
}

fn split_pizza_modifiers() -> Modifiers {
    let meats = Prompt::new(
        component_id("01MEATS"),
        "Meats",
        None,
        vec![Rule::Max(5)],
        Vec::new(),
        vec![
            Choice::new(
                component_id("01PEPPER"),
                "Pepperoni",
                Vec::new(),
                Vec::new(),
            )
            .unwrap()
            .with_price(ChoicePrice::flat_amount(usd(200)).unwrap())
            .with_modifiers(placement_modifiers()),
            Choice::new(component_id("01BACON"), "Bacon", Vec::new(), Vec::new())
                .unwrap()
                .with_price(ChoicePrice::flat_amount(usd(300)).unwrap())
                .with_modifiers(placement_modifiers()),
        ],
    )
    .unwrap();

    Modifiers::new(vec![meats])
}

fn deeply_nested_modifiers() -> Modifiers {
    let amount = Modifiers::new(vec![
        Prompt::new(
            component_id("01AMOUNT"),
            "Amount",
            None,
            vec![Rule::Min(1), Rule::Max(1)],
            Vec::new(),
            vec![
                Choice::new(component_id("01LIGHT"), "Light", Vec::new(), Vec::new()).unwrap(),
                Choice::new(component_id("01EXTRA"), "Extra", Vec::new(), Vec::new()).unwrap(),
            ],
        )
        .unwrap(),
    ]);
    let placement = Modifiers::new(vec![
        Prompt::new(
            component_id("01PLACE"),
            "Placement",
            None,
            vec![Rule::Min(1), Rule::Max(1)],
            Vec::new(),
            vec![
                Choice::new(component_id("01LEFT"), "Left side", Vec::new(), Vec::new())
                    .unwrap()
                    .with_price(ChoicePrice::from_factor(Rate::percent(50)))
                    .with_modifiers(amount),
                Choice::new(
                    component_id("01RIGHT"),
                    "Right side",
                    Vec::new(),
                    Vec::new(),
                )
                .unwrap(),
            ],
        )
        .unwrap(),
    ]);
    let toppings = Prompt::new(
        component_id("01TOPPING"),
        "Toppings",
        None,
        vec![Rule::Max(5)],
        Vec::new(),
        vec![
            Choice::new(
                component_id("01PEPPER"),
                "Pepperoni",
                Vec::new(),
                Vec::new(),
            )
            .unwrap()
            .with_price(ChoicePrice::flat_amount(usd(200)).unwrap())
            .with_modifiers(placement),
        ],
    )
    .unwrap();

    Modifiers::new(vec![toppings])
}

fn placement_modifiers() -> Modifiers {
    Modifiers::new(vec![
        Prompt::new(
            component_id("01PLACE"),
            "Placement",
            None,
            vec![Rule::Min(1), Rule::Max(1)],
            Vec::new(),
            vec![
                Choice::new(component_id("01LEFT"), "Left side", Vec::new(), Vec::new())
                    .unwrap()
                    .with_price(ChoicePrice::from_factor(Rate::percent(50))),
                Choice::new(
                    component_id("01RIGHT"),
                    "Right side",
                    Vec::new(),
                    Vec::new(),
                )
                .unwrap()
                .with_price(ChoicePrice::from_factor(Rate::percent(50))),
                Choice::new(
                    component_id("01WHOLE"),
                    "Whole pizza",
                    Vec::new(),
                    Vec::new(),
                )
                .unwrap()
                .with_price(ChoicePrice::from_factor(Rate::percent(100))),
            ],
        )
        .unwrap(),
    ])
}

fn deeply_nested_priced_modifiers() -> Modifiers {
    let amount = Modifiers::new(vec![
        Prompt::new(
            component_id("01AMOUNT"),
            "Amount",
            None,
            vec![Rule::Min(1), Rule::Max(1)],
            Vec::new(),
            vec![
                Choice::new(component_id("01HALF"), "Half", Vec::new(), Vec::new())
                    .unwrap()
                    .with_price(ChoicePrice::from_factor(Rate::percent(50))),
            ],
        )
        .unwrap(),
    ]);
    let placement = Modifiers::new(vec![
        Prompt::new(
            component_id("01PLACE"),
            "Placement",
            None,
            vec![Rule::Min(1), Rule::Max(1)],
            Vec::new(),
            vec![
                Choice::new(component_id("01LEFT"), "Left side", Vec::new(), Vec::new())
                    .unwrap()
                    .with_price(ChoicePrice::from_factor(Rate::percent(50)))
                    .with_modifiers(amount),
            ],
        )
        .unwrap(),
    ]);

    Modifiers::new(vec![
        Prompt::new(
            component_id("01TOPPING"),
            "Toppings",
            None,
            vec![Rule::Max(5)],
            Vec::new(),
            vec![
                Choice::new(
                    component_id("01PEPPER"),
                    "Pepperoni",
                    Vec::new(),
                    Vec::new(),
                )
                .unwrap()
                .with_price(ChoicePrice::flat_amount(usd(200)).unwrap())
                .with_modifiers(placement),
            ],
        )
        .unwrap(),
    ])
}

fn scheduled_choice_modifiers(defaulted: bool) -> Modifiers {
    let mut rules = vec![Rule::Max(1)];

    if defaulted {
        rules.push(Rule::Default(1));
    }

    Modifiers::new(vec![
        Prompt::new(
            component_id("01SPECIAL"),
            "Specials",
            None,
            vec![Rule::Min(1), Rule::Max(1)],
            Vec::new(),
            vec![
                Choice::new(
                    component_id("01BRUNCH"),
                    "Brunch special",
                    rules,
                    Vec::new(),
                )
                .unwrap()
                .with_schedule(weekday_lunch_schedule()),
            ],
        )
        .unwrap(),
    ])
}

fn weekday_lunch_schedule() -> Schedule {
    Schedule::with_windows(vec![
        ScheduleWindow::new()
            .with_days_of_week(DaysOfWeek::weekdays())
            .with_time_range(LocalTimeRange::from_seconds(11 * 3_600, 14 * 3_600).unwrap()),
    ])
    .unwrap()
}

fn evaluation_time(
    unix_millis: i64,
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
) -> EvaluationTime {
    EvaluationTime::new(
        UtcTime::from_unix_millis(unix_millis),
        CalendarMoment::new(
            LogicalDate::new(year, month, day).unwrap(),
            LocalTimeOfDay::from_hms(hour, minute, second).unwrap(),
            TimeZone::utc(),
        ),
    )
}

fn usd(amount_minor: i64) -> Money {
    Money::new(amount_minor, CurrencyCode::parse("USD").unwrap())
}

fn component_id(suffix: &str) -> ComponentId {
    ComponentId::from_suffix(suffix).unwrap()
}

fn consumer_attribute_id(suffix: &str) -> ConsumerAttributeId {
    ConsumerAttributeId::from_suffix(suffix).unwrap()
}

fn label_id(suffix: &str) -> LabelId {
    LabelId::from_suffix(suffix).unwrap()
}

fn media_id(suffix: &str) -> MediaId {
    MediaId::from_suffix(suffix).unwrap()
}

fn mime(value: &str) -> MediaMimeType {
    MediaMimeType::parse(value).unwrap()
}

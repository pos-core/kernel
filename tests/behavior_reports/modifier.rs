use pos_core_kernel::prelude::*;

use crate::support::behavior::*;
use crate::support::md_report::{DefinitionLink, ModuleReport};

pub fn report() -> ModuleReport {
    ModuleReport {
        slug: "modifier",
        title: "Modifier",
        description: "Described behavior tests for prompts, choices, choice inputs, rules, hydration, dehydration, and pricing.",
        definitions: vec![
            DefinitionLink::new("Modifier definitions", "../src/modifier/modifiers.md"),
            DefinitionLink::new("Choice inputs", "../src/modifier/choice-inputs.md"),
            DefinitionLink::new("Selections", "../src/modifier/selections.md"),
            DefinitionLink::new("Configuration", "../src/modifier/configuration.md"),
            DefinitionLink::new(
                "Configuration snapshot",
                "../src/modifier/configuration-snapshot.md",
            ),
            DefinitionLink::new("Modifier pricing", "../src/modifier/modifier-pricing.md"),
        ],
        cases: vec![
            DEFAULTS_DEHYDRATE_INTO_EFFECTIVE_SELECTIONS.report_case(),
            CONFIGURATION_SNAPSHOT_PRESERVES_DEFAULT_SELECTIONS_LABELS_AND_PRICE_FACTS
                .report_case(),
            DEFAULTS_MUST_SATISFY_PROMPT_MIN_AND_MAX_RULES.report_case(),
            EXPLICIT_SELECTIONS_REPLACE_DEFAULTS_FOR_A_PROMPT.report_case(),
            PROMPT_MIN_AND_MAX_RULES_VALIDATE_SELECTION_COUNTS.report_case(),
            MAX_ZERO_PROMPT_ALLOWS_NO_SELECTION_AND_REJECTS_ANY_SELECTION.report_case(),
            OPTIONAL_PROMPT_WITHOUT_SELECTION_HYDRATES_EMPTY_CONFIGURATION.report_case(),
            SCHEDULED_CHOICE_REQUIRES_EVALUATION_TIME_WHEN_SELECTED.report_case(),
            SCHEDULED_CHOICE_HYDRATES_ONLY_INSIDE_ITS_SCHEDULE.report_case(),
            SCHEDULED_CHOICE_IS_NOT_VISIBLE_OUTSIDE_ITS_SCHEDULE.report_case(),
            SCHEDULED_DEFAULT_CHOICE_USES_SUPPLIED_EVALUATION_TIME.report_case(),
            CHOICE_RULES_APPLY_TO_SELECTED_CHOICE_QUANTITY.report_case(),
            CHOICE_INPUTS_COLLECT_ONCE_OR_ONCE_PER_SELECTED_UNIT_AND_ROUND_TRIP.report_case(),
            CHOICE_INPUTS_REJECT_INVALID_DEFINITIONS_AND_VALUES.report_case(),
            CHOICE_DEFAULT_RULES_ARE_UNIQUE_NONZERO_AND_WITHIN_CHOICE_BOUNDS.report_case(),
            DUPLICATE_CHOICE_SELECTIONS_ARE_REJECTED_EVEN_WITH_DIFFERENT_NESTED_SELECTIONS
                .report_case(),
            PROMPT_REJECTS_ZERO_DUPLICATE_AND_UNKNOWN_CHOICE_SELECTIONS.report_case(),
            PROMPT_REJECTS_SELECTION_COUNT_OVERFLOW.report_case(),
            DUPLICATE_MIN_OR_MAX_RULES_ARE_REJECTED.report_case(),
            DEFINITIONS_REJECT_EMPTY_TITLES_INVALID_PROMPT_RULES_AND_EMPTY_REQUIRED_PROMPTS
                .report_case(),
            DEFINITIONS_REJECT_INVALID_MIN_MAX_CONSTRAINTS_AND_DUPLICATE_CHOICE_DEFINITIONS
                .report_case(),
            CONFIGURATION_DEHYDRATES_NESTED_CHOICES_AND_ROUND_TRIPS.report_case(),
            THREE_LEVEL_NESTED_SELECTION_DEHYDRATES_AND_ROUND_TRIPS.report_case(),
            NESTED_REQUIRED_PROMPTS_MUST_BE_SATISFIED_WHEN_PARENT_CHOICE_IS_SELECTED.report_case(),
            NESTED_DEFAULTS_DEHYDRATE_INTO_EFFECTIVE_SELECTIONS.report_case(),
            SAME_MODIFIERS_CAN_HOLD_REPEATED_PROMPT_IDS_AS_ORDERED_INSTANCES.report_case(),
            SAME_PROMPT_AND_CHOICE_IDS_CAN_BE_REUSED_IN_DIFFERENT_BRANCHES.report_case(),
            STRICT_HYDRATE_REJECTS_UNKNOWN_PROMPT_SELECTIONS_AND_EXTRA_PROMPT_OCCURRENCES
                .report_case(),
            STRICT_HYDRATE_REJECTS_NESTED_SELECTIONS_FOR_TERMINAL_CHOICES.report_case(),
            TITLED_MODIFIERS_EXPOSE_TITLE_WITHOUT_AFFECTING_HYDRATION.report_case(),
            MODIFIERS_CAN_WALK_NESTED_DEFINITION_TREE.report_case(),
            PROMPT_VALIDATION_RETURNS_EFFECTS_AND_NESTED_MODIFIER_DEFINITIONS.report_case(),
            FLAT_MODIFIER_PRICE_IS_MULTIPLIED_BY_SELECTED_FACTOR.report_case(),
            INVARIANT_RATE_PRICE_ADDS_TO_FLAT_AMOUNT_WITHOUT_FLOATS.report_case(),
            NESTED_FACTORS_MULTIPLY_UP_TO_THE_NEAREST_PRICED_ANCESTOR.report_case(),
            DEFAULTS_ARE_FREE_UNLESS_PRICING_POLICY_SAYS_OTHERWISE.report_case(),
            UNCONSUMED_ROOT_FACTOR_IS_INVALID.report_case(),
        ],
    }
}

pub const DEFAULTS_DEHYDRATE_INTO_EFFECTIVE_SELECTIONS: DescribedBehavior = DescribedBehavior::new(
    "defaults dehydrate into effective selections",
    "Default choices hydrate into configuration and dehydrate into effective selections with their default source preserved.",
    defaults_dehydrate_into_effective_selections,
);

#[test]
fn defaults_dehydrate_into_effective_selections() {
    let cheese_id = component_id("01CHEESE");
    let american_id = component_id("01AMERICAN");
    let modifiers = Modifiers::new(vec![cheese_prompt_with_default(
        cheese_id.clone(),
        american_id.clone(),
    )]);

    let configuration = modifiers.hydrate(&Selections::new()).unwrap();
    let dehydrated = configuration.dehydrate();
    let expected = Selections::new().with_prompt(
        cheese_id.clone(),
        vec![ChoiceSelection::new(american_id.clone(), 1).with_source(SelectionSource::Default)],
    );

    assert_eq!(dehydrated, expected);
    assert_eq!(
        configuration.prompt(&cheese_id).unwrap().choices()[0].choice_id(),
        &american_id
    );
    assert_eq!(modifiers.hydrate(&dehydrated).unwrap(), configuration);
}

pub const CONFIGURATION_SNAPSHOT_PRESERVES_DEFAULT_SELECTIONS_LABELS_AND_PRICE_FACTS:
    DescribedBehavior = DescribedBehavior::new(
    "configuration snapshot preserves default selections labels and price facts",
    "A configuration snapshot includes default selections, prompt and choice labels, price definitions, price contributions, and the modifier total.",
    configuration_snapshot_preserves_default_selections_labels_and_price_facts,
);

#[test]
fn configuration_snapshot_preserves_default_selections_labels_and_price_facts() {
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
    assert_eq!(prompt.description(), Some("Choose the included cheese"));
    assert_eq!(choice.choice_id(), &american_id);
    assert_eq!(choice.title(), "American");
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

pub const DEFAULTS_MUST_SATISFY_PROMPT_MIN_AND_MAX_RULES: DescribedBehavior =
    DescribedBehavior::new(
        "defaults must satisfy prompt min and max rules",
        "Choice defaults are validated against the containing prompt selection count rules.",
        defaults_must_satisfy_prompt_min_and_max_rules,
    );

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

pub const EXPLICIT_SELECTIONS_REPLACE_DEFAULTS_FOR_A_PROMPT: DescribedBehavior =
    DescribedBehavior::new(
        "explicit selections replace defaults for a prompt",
        "An explicit selection suppresses default choices on the same prompt instead of merging with them.",
        explicit_selections_replace_defaults_for_a_prompt,
    );

#[test]
fn explicit_selections_replace_defaults_for_a_prompt() {
    let cheese_id = component_id("01CHEESE");
    let swiss_id = component_id("01SWISS");
    let modifiers = Modifiers::new(vec![cheese_prompt_with_default(
        cheese_id.clone(),
        component_id("01AMERICAN"),
    )]);
    let selections = Selections::new().with_prompt(
        cheese_id.clone(),
        vec![ChoiceSelection::new(swiss_id.clone(), 1)],
    );

    let configuration = modifiers.hydrate(&selections).unwrap();
    let choices = configuration.prompt(&cheese_id).unwrap().choices();

    assert_eq!(choices.len(), 1);
    assert_eq!(choices[0].choice_id(), &swiss_id);
    assert_eq!(choices[0].source(), SelectionSource::Explicit);
}

pub const PROMPT_MIN_AND_MAX_RULES_VALIDATE_SELECTION_COUNTS: DescribedBehavior =
    DescribedBehavior::new(
        "prompt min and max rules validate selection counts",
        "Prompt Min and Max rules validate the summed selected counts across choices.",
        prompt_min_and_max_rules_validate_selection_counts,
    );

#[test]
fn prompt_min_and_max_rules_validate_selection_counts() {
    let cheddar_id = component_id("01CHEDDAR");
    let swiss_id = component_id("01SWISS");
    let american_id = component_id("01AMERICAN");
    let prompt = Prompt::new(
        component_id("01CHEESE"),
        "Cheese",
        None,
        vec![Rule::Min(1), Rule::Max(2)],
        Vec::new(),
        vec![
            Choice::new(cheddar_id.clone(), "Cheddar", Vec::new(), Vec::new()).unwrap(),
            Choice::new(swiss_id.clone(), "Swiss", Vec::new(), Vec::new()).unwrap(),
            Choice::new(american_id.clone(), "American", Vec::new(), Vec::new()).unwrap(),
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
                ChoiceSelection::new(american_id.clone(), 1),
            ])
            .is_ok()
    );
    assert_eq!(
        prompt.validate_selections(&[
            ChoiceSelection::new(component_id("01CHEDDAR"), 1),
            ChoiceSelection::new(component_id("01SWISS"), 1),
            ChoiceSelection::new(american_id, 1),
        ]),
        Err(ModifierError::AboveMaximum {
            max_select: 2,
            actual: 3
        })
    );
}

pub const MAX_ZERO_PROMPT_ALLOWS_NO_SELECTION_AND_REJECTS_ANY_SELECTION: DescribedBehavior =
    DescribedBehavior::new(
        "max zero prompt allows no selection and rejects any selection",
        "A prompt with Max(0) is valid when empty and invalid when any choice is selected.",
        max_zero_prompt_allows_no_selection_and_rejects_any_selection,
    );

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

pub const OPTIONAL_PROMPT_WITHOUT_SELECTION_HYDRATES_EMPTY_CONFIGURATION: DescribedBehavior =
    DescribedBehavior::new(
        "optional prompt without selection hydrates empty configuration",
        "An optional prompt with no selected choices still appears in hydrated configuration with an empty choice list.",
        optional_prompt_without_selection_hydrates_empty_configuration,
    );

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

pub const SCHEDULED_CHOICE_REQUIRES_EVALUATION_TIME_WHEN_SELECTED: DescribedBehavior =
    DescribedBehavior::new(
        "scheduled choice requires evaluation time when selected",
        "A selected choice with a schedule cannot be hydrated unless the caller supplies explicit EvaluationTime.",
        scheduled_choice_requires_evaluation_time_when_selected,
    );

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

pub const SCHEDULED_CHOICE_HYDRATES_ONLY_INSIDE_ITS_SCHEDULE: DescribedBehavior =
    DescribedBehavior::new(
        "scheduled choice hydrates only inside its schedule",
        "A scheduled choice accepts selections inside its own schedule and rejects selections outside it.",
        scheduled_choice_hydrates_only_inside_its_schedule,
    );

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

pub const SCHEDULED_CHOICE_IS_NOT_VISIBLE_OUTSIDE_ITS_SCHEDULE: DescribedBehavior =
    DescribedBehavior::new(
        "scheduled choice is not visible outside its schedule",
        "A scheduled choice is filtered from visible modifier traversal when EvaluationTime falls outside its schedule.",
        scheduled_choice_is_not_visible_outside_its_schedule,
    );

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

pub const SCHEDULED_DEFAULT_CHOICE_USES_SUPPLIED_EVALUATION_TIME: DescribedBehavior =
    DescribedBehavior::new(
        "scheduled default choice uses supplied evaluation time",
        "A scheduled default choice requires EvaluationTime and only defaults when its schedule includes that time.",
        scheduled_default_choice_uses_supplied_evaluation_time,
    );

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

pub const CHOICE_RULES_APPLY_TO_SELECTED_CHOICE_QUANTITY: DescribedBehavior =
    DescribedBehavior::new(
        "choice rules apply to selected choice quantity",
        "Choice-level Min and Max rules validate the selected count on that choice.",
        choice_rules_apply_to_selected_choice_quantity,
    );

#[test]
fn choice_rules_apply_to_selected_choice_quantity() {
    let modifiers = pizza_modifiers();
    let toppings_id = component_id("01TOPPING");
    let pepperoni_id = component_id("01PEPPER");

    let valid = Selections::new().with_prompt(
        toppings_id.clone(),
        vec![
            ChoiceSelection::new(pepperoni_id.clone(), 1).with_modifiers(
                Selections::new().with_prompt(
                    component_id("01PLACE"),
                    vec![ChoiceSelection::new(component_id("01LEFT"), 1)],
                ),
            ),
        ],
    );
    assert!(modifiers.hydrate(&valid).is_ok());

    let invalid = Selections::new().with_prompt(
        toppings_id,
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
        modifiers.hydrate(&invalid),
        Err(ModifierError::ChoiceAboveMaximum {
            choice_id: pepperoni_id,
            max_select: 1,
            actual: 2
        })
    );
}

pub const CHOICE_DEFAULT_RULES_ARE_UNIQUE_NONZERO_AND_WITHIN_CHOICE_BOUNDS: DescribedBehavior =
    DescribedBehavior::new(
        "choice default rules are unique nonzero and within choice bounds",
        "A choice default must be unique, greater than zero, and valid against the choice's own Min and Max rules.",
        choice_default_rules_are_unique_nonzero_and_within_choice_bounds,
    );

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

pub const CHOICE_INPUTS_COLLECT_ONCE_OR_ONCE_PER_SELECTED_UNIT_AND_ROUND_TRIP: DescribedBehavior =
    DescribedBehavior::new(
        "choice inputs collect once or once per selected unit and round trip",
        "A selected choice can collect one text value for the whole selection and ordered text values for individual units, preserving them through hydration, effective selections, and configuration snapshots.",
        choice_inputs_collect_once_or_once_per_selected_unit_and_round_trip,
    );

#[test]
fn choice_inputs_collect_once_or_once_per_selected_unit_and_round_trip() {
    let prompt_id = component_id("01CUPCAKES");
    let choice_id = component_id("01CUPCAKE");
    let request_id = component_id("01REQUEST");
    let name_id = component_id("01NAME");
    let modifiers = Modifiers::new(vec![
        Prompt::new(
            prompt_id.clone(),
            "How many cupcakes do you want?",
            None,
            vec![Rule::Min(1), Rule::Max(12)],
            Vec::new(),
            vec![
                Choice::new(
                    choice_id.clone(),
                    "Cupcake",
                    vec![Rule::Max(12)],
                    Vec::new(),
                )
                .unwrap()
                .with_inputs(vec![
                    ChoiceInput::new(
                        request_id.clone(),
                        "Any special requests?",
                        false,
                        None,
                        Some(100),
                        false,
                    )
                    .unwrap(),
                    ChoiceInput::new(name_id.clone(), "Name", true, Some(1), Some(50), true)
                        .unwrap(),
                ])
                .unwrap(),
            ],
        )
        .unwrap(),
    ]);
    let selections = Selections::new().with_prompt(
        prompt_id.clone(),
        vec![ChoiceSelection::new(choice_id, 3).with_inputs(vec![
            ChoiceInputValue::for_unit(name_id.clone(), 2, "Bob"),
            ChoiceInputValue::once(request_id.clone(), "Use blue frosting"),
            ChoiceInputValue::for_unit(name_id.clone(), 1, "Alice"),
            ChoiceInputValue::for_unit(name_id.clone(), 3, "Carol"),
        ])],
    );

    let configuration = modifiers.hydrate(&selections).unwrap();
    let inputs = configuration.prompt(&prompt_id).unwrap().choices()[0].inputs();

    assert_eq!(inputs.len(), 4);
    assert_eq!(inputs[0].input_id(), &request_id);
    assert_eq!(inputs[0].title(), "Any special requests?");
    assert_eq!(inputs[0].unit(), None);
    assert_eq!(inputs[0].value(), "Use blue frosting");
    assert_eq!(inputs[1].input_id(), &name_id);
    assert_eq!(inputs[1].unit(), Some(1));
    assert_eq!(inputs[1].value(), "Alice");
    assert_eq!(inputs[2].unit(), Some(2));
    assert_eq!(inputs[2].value(), "Bob");
    assert_eq!(inputs[3].unit(), Some(3));
    assert_eq!(inputs[3].value(), "Carol");

    let effective = configuration.dehydrate();
    assert_eq!(modifiers.hydrate(&effective).unwrap(), configuration);

    let snapshot = configuration
        .snapshot(&usd(0), ModifierPricingPolicy::default())
        .unwrap();
    let snapshotted_inputs = snapshot.prompt(&prompt_id).unwrap().choices()[0].inputs();
    assert_eq!(snapshotted_inputs[0].title(), "Any special requests?");
    assert_eq!(snapshotted_inputs[0].value(), "Use blue frosting");
    assert_eq!(snapshotted_inputs[3].unit(), Some(3));
    assert_eq!(snapshotted_inputs[3].value(), "Carol");
}

pub const CHOICE_INPUTS_REJECT_INVALID_DEFINITIONS_AND_VALUES: DescribedBehavior =
    DescribedBehavior::new(
        "choice inputs reject invalid definitions and values",
        "Choice input authoring requires labels, coherent length bounds, and unique IDs; hydration enforces known inputs, once-versus-per-unit occurrence, required values, and character-length limits.",
        choice_inputs_reject_invalid_definitions_and_values,
    );

#[test]
fn choice_inputs_reject_invalid_definitions_and_values() {
    let choice_id = component_id("01CUPCAKE");
    let request_id = component_id("01REQUEST");
    let name_id = component_id("01NAME");

    assert_eq!(
        ChoiceInput::new(component_id("01EMPTY"), " ", false, None, None, false,),
        Err(choice_input_error(ChoiceInputError::EmptyTitle))
    );
    assert_eq!(
        ChoiceInput::new(name_id.clone(), "Name", true, Some(5), Some(4), true,),
        Err(choice_input_error(
            ChoiceInputError::InvalidLengthConstraints {
                input_id: name_id.clone(),
                min_length: 5,
                max_length: 4,
            }
        ))
    );

    let request = ChoiceInput::new(
        request_id.clone(),
        "Any special requests?",
        false,
        None,
        Some(10),
        false,
    )
    .unwrap();
    let name = ChoiceInput::new(name_id.clone(), "Name", true, Some(1), Some(5), true).unwrap();

    assert_eq!(
        Choice::new(choice_id.clone(), "Cupcake", Vec::new(), Vec::new())
            .unwrap()
            .with_inputs(vec![request.clone(), request.clone()]),
        Err(choice_input_error(ChoiceInputError::DuplicateDefinition(
            request_id.clone()
        )))
    );
    let prompt = Prompt::new(
        component_id("01CUPCAKES"),
        "How many cupcakes do you want?",
        None,
        vec![Rule::Max(12)],
        Vec::new(),
        vec![
            Choice::new(
                choice_id.clone(),
                "Cupcake",
                vec![Rule::Max(12)],
                Vec::new(),
            )
            .unwrap()
            .with_inputs(vec![request, name])
            .unwrap(),
        ],
    )
    .unwrap();

    let validate = |inputs| {
        prompt
            .validate_selections(&[ChoiceSelection::new(choice_id.clone(), 2).with_inputs(inputs)])
    };

    assert_eq!(
        validate(vec![ChoiceInputValue::for_unit(
            name_id.clone(),
            1,
            "Alice",
        )]),
        Err(choice_input_error(ChoiceInputError::MissingRequiredValue {
            choice_id: choice_id.clone(),
            input_id: name_id.clone(),
            expected: 2,
            actual: 1,
        }))
    );
    assert_eq!(
        validate(vec![ChoiceInputValue::once(name_id.clone(), "Alice")]),
        Err(choice_input_error(ChoiceInputError::UnitRequired {
            choice_id: choice_id.clone(),
            input_id: name_id.clone(),
        }))
    );
    assert_eq!(
        validate(vec![ChoiceInputValue::for_unit(
            request_id.clone(),
            1,
            "No nuts",
        )]),
        Err(choice_input_error(ChoiceInputError::UnexpectedUnit {
            choice_id: choice_id.clone(),
            input_id: request_id.clone(),
        }))
    );
    assert_eq!(
        validate(vec![ChoiceInputValue::for_unit(
            name_id.clone(),
            3,
            "Alice",
        )]),
        Err(choice_input_error(ChoiceInputError::UnitOutOfRange {
            choice_id: choice_id.clone(),
            input_id: name_id.clone(),
            unit: 3,
            quantity: 2,
        }))
    );
    assert_eq!(
        validate(vec![
            ChoiceInputValue::for_unit(name_id.clone(), 1, "Alice"),
            ChoiceInputValue::for_unit(name_id.clone(), 1, "Alice"),
        ]),
        Err(choice_input_error(ChoiceInputError::DuplicateValue {
            choice_id: choice_id.clone(),
            input_id: name_id.clone(),
            unit: Some(1),
        }))
    );
    assert_eq!(
        validate(vec![ChoiceInputValue::for_unit(name_id.clone(), 1, "")]),
        Err(choice_input_error(ChoiceInputError::BelowMinimumLength {
            choice_id: choice_id.clone(),
            input_id: name_id.clone(),
            min_length: 1,
            actual: 0,
        }))
    );
    assert_eq!(
        validate(vec![ChoiceInputValue::once(
            request_id.clone(),
            "12345678901",
        )]),
        Err(choice_input_error(ChoiceInputError::AboveMaximumLength {
            choice_id: choice_id.clone(),
            input_id: request_id,
            max_length: 10,
            actual: 11,
        }))
    );
    assert_eq!(
        validate(vec![ChoiceInputValue::once(
            component_id("01UNKNOWN"),
            "Unknown",
        )]),
        Err(choice_input_error(ChoiceInputError::UnknownInput {
            choice_id,
            input_id: component_id("01UNKNOWN"),
        }))
    );
}

pub const DUPLICATE_CHOICE_SELECTIONS_ARE_REJECTED_EVEN_WITH_DIFFERENT_NESTED_SELECTIONS:
    DescribedBehavior = DescribedBehavior::new(
    "duplicate choice selections are rejected even with different nested selections",
    "A prompt cannot select the same choice ID twice; distinct configuration must be modeled below the selected choice.",
    duplicate_choice_selections_are_rejected_even_with_different_nested_selections,
);

#[test]
fn duplicate_choice_selections_are_rejected_even_with_different_nested_selections() {
    let modifiers = pizza_modifiers();
    let toppings_id = component_id("01TOPPING");
    let pepperoni_id = component_id("01PEPPER");
    let placement_id = component_id("01PLACE");
    let selections = Selections::new().with_prompt(
        toppings_id,
        vec![
            ChoiceSelection::new(pepperoni_id.clone(), 1).with_modifiers(
                Selections::new().with_prompt(
                    placement_id.clone(),
                    vec![ChoiceSelection::new(component_id("01LEFT"), 1)],
                ),
            ),
            ChoiceSelection::new(pepperoni_id.clone(), 1).with_modifiers(
                Selections::new().with_prompt(
                    placement_id,
                    vec![ChoiceSelection::new(component_id("01RIGHT"), 1)],
                ),
            ),
        ],
    );

    assert_eq!(
        modifiers.hydrate(&selections),
        Err(ModifierError::DuplicateSelection(pepperoni_id))
    );
}

pub const PROMPT_REJECTS_ZERO_DUPLICATE_AND_UNKNOWN_CHOICE_SELECTIONS: DescribedBehavior =
    DescribedBehavior::new(
        "prompt rejects zero duplicate and unknown choice selections",
        "Prompt validation rejects zero counts, repeated choice IDs, and choice IDs that do not exist under the prompt.",
        prompt_rejects_zero_duplicate_and_unknown_choice_selections,
    );

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
        Err(ModifierError::DuplicateSelection(ranch_id))
    );
    assert_eq!(
        prompt.validate_selections(&[ChoiceSelection::new(component_id("01HOT"), 1)]),
        Err(ModifierError::UnknownSelection(component_id("01HOT")))
    );
}

pub const PROMPT_REJECTS_SELECTION_COUNT_OVERFLOW: DescribedBehavior = DescribedBehavior::new(
    "prompt rejects selection count overflow",
    "Prompt validation uses checked arithmetic while summing selected counts.",
    prompt_rejects_selection_count_overflow,
);

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

pub const DUPLICATE_MIN_OR_MAX_RULES_ARE_REJECTED: DescribedBehavior = DescribedBehavior::new(
    "duplicate min or max rules are rejected",
    "Each rule kind can appear at most once in a prompt or choice rule set.",
    duplicate_min_or_max_rules_are_rejected,
);

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

pub const DEFINITIONS_REJECT_EMPTY_TITLES_INVALID_PROMPT_RULES_AND_EMPTY_REQUIRED_PROMPTS:
    DescribedBehavior = DescribedBehavior::new(
    "definitions reject empty titles invalid prompt rules and empty required prompts",
    "Definition construction validates prompt and choice titles, rejects prompt defaults, and rejects required prompts with no choices.",
    definitions_reject_empty_titles_invalid_prompt_rules_and_empty_required_prompts,
);

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

pub const DEFINITIONS_REJECT_INVALID_MIN_MAX_CONSTRAINTS_AND_DUPLICATE_CHOICE_DEFINITIONS:
    DescribedBehavior = DescribedBehavior::new(
    "definitions reject invalid min max constraints and duplicate choice definitions",
    "Definition construction rejects Min greater than Max and duplicate choice IDs inside a prompt definition.",
    definitions_reject_invalid_min_max_constraints_and_duplicate_choice_definitions,
);

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

pub const CONFIGURATION_DEHYDRATES_NESTED_CHOICES_AND_ROUND_TRIPS: DescribedBehavior =
    DescribedBehavior::new(
        "configuration dehydrates nested choices and round trips",
        "Explicit nested choices survive hydrate, dehydrate, and rehydrate without changing configuration.",
        configuration_dehydrates_nested_choices_and_round_trips,
    );

#[test]
fn configuration_dehydrates_nested_choices_and_round_trips() {
    let modifiers = pizza_modifiers_with_mushrooms();
    let toppings_id = component_id("01TOPPING");
    let placement_id = component_id("01PLACE");
    let selections = Selections::new().with_prompt(
        toppings_id,
        vec![
            ChoiceSelection::new(component_id("01PEPPER"), 1).with_modifiers(
                Selections::new().with_prompt(
                    placement_id.clone(),
                    vec![ChoiceSelection::new(component_id("01LEFT"), 1)],
                ),
            ),
            ChoiceSelection::new(component_id("01MUSH"), 1).with_modifiers(
                Selections::new().with_prompt(
                    placement_id,
                    vec![ChoiceSelection::new(component_id("01WHOLE"), 1)],
                ),
            ),
        ],
    );

    let configuration = modifiers.hydrate(&selections).unwrap();
    let dehydrated = configuration.dehydrate();

    assert_eq!(dehydrated, selections);
    assert_eq!(modifiers.hydrate(&dehydrated).unwrap(), configuration);
}

pub const THREE_LEVEL_NESTED_SELECTION_DEHYDRATES_AND_ROUND_TRIPS: DescribedBehavior =
    DescribedBehavior::new(
        "three level nested selection dehydrates and round trips",
        "Deeply nested explicit selections remain stable across hydrate and dehydrate.",
        three_level_nested_selection_dehydrates_and_round_trips,
    );

#[test]
fn three_level_nested_selection_dehydrates_and_round_trips() {
    let modifiers = deeply_nested_modifiers();
    let selections = Selections::new().with_prompt(
        component_id("01TOPPING"),
        vec![
            ChoiceSelection::new(component_id("01PEPPER"), 1).with_modifiers(
                Selections::new().with_prompt(
                    component_id("01PLACE"),
                    vec![
                        ChoiceSelection::new(component_id("01LEFT"), 1).with_modifiers(
                            Selections::new().with_prompt(
                                component_id("01AMOUNT"),
                                vec![ChoiceSelection::new(component_id("01EXTRA"), 1)],
                            ),
                        ),
                    ],
                ),
            ),
        ],
    );

    let configuration = modifiers.hydrate(&selections).unwrap();
    let dehydrated = configuration.dehydrate();

    assert_eq!(dehydrated, selections);
    assert_eq!(modifiers.hydrate(&dehydrated).unwrap(), configuration);
}

pub const NESTED_REQUIRED_PROMPTS_MUST_BE_SATISFIED_WHEN_PARENT_CHOICE_IS_SELECTED:
    DescribedBehavior = DescribedBehavior::new(
    "nested required prompts must be satisfied when parent choice is selected",
    "Selecting a choice with required nested modifiers validates the nested required prompt immediately.",
    nested_required_prompts_must_be_satisfied_when_parent_choice_is_selected,
);

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

pub const NESTED_DEFAULTS_DEHYDRATE_INTO_EFFECTIVE_SELECTIONS: DescribedBehavior =
    DescribedBehavior::new(
        "nested defaults dehydrate into effective selections",
        "Nested defaults under an explicit parent choice are included in dehydrated effective selections with their default source preserved.",
        nested_defaults_dehydrate_into_effective_selections,
    );

#[test]
fn nested_defaults_dehydrate_into_effective_selections() {
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

pub const SAME_MODIFIERS_CAN_HOLD_REPEATED_PROMPT_IDS_AS_ORDERED_INSTANCES: DescribedBehavior =
    DescribedBehavior::new(
        "same modifiers can hold repeated prompt IDs as ordered instances",
        "Repeated prompt IDs in one modifier container hydrate by occurrence and dehydrate back as ordered prompt instances.",
        same_modifiers_can_hold_repeated_prompt_ids_as_ordered_instances,
    );

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
            vec![Choice::new(hot_id.clone(), "Hot", Vec::new(), Vec::new()).unwrap()],
        )
        .unwrap(),
        Prompt::new(
            sauce_id.clone(),
            "Dipping sauce",
            None,
            vec![Rule::Min(1), Rule::Max(1)],
            Vec::new(),
            vec![Choice::new(ranch_id.clone(), "Ranch", Vec::new(), Vec::new()).unwrap()],
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
            vec![ChoiceSelection::new(ranch_id.clone(), 1)],
        ));

    let configuration = modifiers.hydrate(&selections).unwrap();

    assert_eq!(
        configuration.prompt_at(&sauce_id, 1).unwrap().choices()[0].choice_id(),
        &ranch_id
    );
    assert_eq!(configuration.dehydrate(), selections);
}

pub const SAME_PROMPT_AND_CHOICE_IDS_CAN_BE_REUSED_IN_DIFFERENT_BRANCHES: DescribedBehavior =
    DescribedBehavior::new(
        "same prompt and choice IDs can be reused in different branches",
        "Prompt and choice IDs can recur in separate branches because nested structure scopes selection lookup.",
        same_prompt_and_choice_ids_can_be_reused_in_different_branches,
    );

#[test]
fn same_prompt_and_choice_ids_can_be_reused_in_different_branches() {
    let modifiers = split_pizza_modifiers();
    let meats_id = component_id("01MEATS");
    let placement_id = component_id("01PLACE");
    let selections = Selections::new().with_prompt(
        meats_id.clone(),
        vec![
            ChoiceSelection::new(component_id("01PEPPER"), 1).with_modifiers(
                Selections::new().with_prompt(
                    placement_id.clone(),
                    vec![ChoiceSelection::new(component_id("01LEFT"), 1)],
                ),
            ),
            ChoiceSelection::new(component_id("01BACON"), 1).with_modifiers(
                Selections::new().with_prompt(
                    placement_id.clone(),
                    vec![ChoiceSelection::new(component_id("01RIGHT"), 1)],
                ),
            ),
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

pub const STRICT_HYDRATE_REJECTS_UNKNOWN_PROMPT_SELECTIONS_AND_EXTRA_PROMPT_OCCURRENCES:
    DescribedBehavior = DescribedBehavior::new(
    "strict hydrate rejects unknown prompt selections and extra prompt occurrences",
    "Strict hydration rejects prompt selections that are not present at the current modifier level, including too many repeated occurrences.",
    strict_hydrate_rejects_unknown_prompt_selections_and_extra_prompt_occurrences,
);

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

pub const STRICT_HYDRATE_REJECTS_NESTED_SELECTIONS_FOR_TERMINAL_CHOICES: DescribedBehavior =
    DescribedBehavior::new(
        "strict hydrate rejects nested selections for terminal choices",
        "Choices without nested modifiers reject unexpected nested selection payloads.",
        strict_hydrate_rejects_nested_selections_for_terminal_choices,
    );

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

pub const TITLED_MODIFIERS_EXPOSE_TITLE_WITHOUT_AFFECTING_HYDRATION: DescribedBehavior =
    DescribedBehavior::new(
        "titled modifiers expose title without affecting hydration",
        "A modifier title is descriptive metadata and does not affect selection hydration.",
        titled_modifiers_expose_title_without_affecting_hydration,
    );

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

pub const MODIFIERS_CAN_WALK_NESTED_DEFINITION_TREE: DescribedBehavior = DescribedBehavior::new(
    "modifiers can walk nested definition tree",
    "The definition walker visits modifiers, prompts, and choices in nested order.",
    modifiers_can_walk_nested_definition_tree,
);

#[test]
fn modifiers_can_walk_nested_definition_tree() {
    let modifiers = pizza_modifiers();
    let mut labels = Vec::new();

    modifiers.walk(&mut |node| match node {
        ModifierNode::Modifiers(_) => labels.push("modifiers".to_owned()),
        ModifierNode::Prompt(prompt) => labels.push(format!("prompt:{}", prompt.title())),
        ModifierNode::Choice(choice) => labels.push(format!("choice:{}", choice.title())),
    });

    assert_eq!(
        labels,
        vec![
            "modifiers",
            "prompt:Toppings",
            "choice:Pepperoni",
            "modifiers",
            "prompt:Placement",
            "choice:Left side",
            "choice:Right side",
            "choice:Whole pizza"
        ]
    );
}

pub const PROMPT_VALIDATION_RETURNS_EFFECTS_AND_NESTED_MODIFIER_DEFINITIONS: DescribedBehavior =
    DescribedBehavior::new(
        "prompt validation returns effects and nested modifier definitions",
        "Validated choice selections carry the choice effects and nested modifier definition needed by consumers.",
        prompt_validation_returns_effects_and_nested_modifier_definitions,
    );

#[test]
fn prompt_validation_returns_effects_and_nested_modifier_definitions() {
    let bacon_id = component_id("01BACON");
    let placement_prompt = placement_prompt(component_id("01PLACE"));
    let nested_modifiers = Modifiers::new(vec![placement_prompt]);
    let effect = Effect::new(
        EffectSource::Choice(bacon_id.clone()),
        EffectTarget::ConfiguredCatalogItem,
        EffectDomain::Price,
        EffectPayload::Standard {
            kind: "add_minor_units".to_owned(),
            value: "200".to_owned(),
        },
    );
    let prompt = Prompt::new(
        component_id("01MEATS"),
        "Meats",
        None,
        vec![Rule::Max(3)],
        Vec::new(),
        vec![
            Choice::new(
                bacon_id.clone(),
                "Bacon",
                vec![Rule::Max(2)],
                vec![effect.clone()],
            )
            .unwrap()
            .with_modifiers(nested_modifiers),
        ],
    )
    .unwrap();

    let selections = prompt
        .validate_selections(&[ChoiceSelection::new(bacon_id.clone(), 2)])
        .unwrap();

    assert_eq!(selections[0].choice_id(), &bacon_id);
    assert_eq!(selections[0].quantity(), 2);
    assert_eq!(selections[0].effects(), &[effect]);
    assert_eq!(
        selections[0].modifiers().unwrap().prompts()[0].title(),
        "Placement"
    );
}

pub const FLAT_MODIFIER_PRICE_IS_MULTIPLIED_BY_SELECTED_FACTOR: DescribedBehavior =
    DescribedBehavior::new(
        "flat modifier price is multiplied by selected factor",
        "A priced choice emits one contribution and selected child factor choices multiply that contribution upward.",
        flat_modifier_price_is_multiplied_by_selected_factor,
    );

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

pub const INVARIANT_RATE_PRICE_ADDS_TO_FLAT_AMOUNT_WITHOUT_FLOATS: DescribedBehavior =
    DescribedBehavior::new(
        "invariant rate price adds to flat amount without floats",
        "A choice can add a flat amount plus an integer rate of the selected variant invariant price.",
        invariant_rate_price_adds_to_flat_amount_without_floats,
    );

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

pub const NESTED_FACTORS_MULTIPLY_UP_TO_THE_NEAREST_PRICED_ANCESTOR: DescribedBehavior =
    DescribedBehavior::new(
        "nested factors multiply up to the nearest priced ancestor",
        "Factor choices can stack through nested modifiers and are consumed by the nearest priced ancestor branch.",
        nested_factors_multiply_up_to_the_nearest_priced_ancestor,
    );

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

pub const DEFAULTS_ARE_FREE_UNLESS_PRICING_POLICY_SAYS_OTHERWISE: DescribedBehavior =
    DescribedBehavior::new(
        "defaults are free unless pricing policy says otherwise",
        "Default selected priced choices contribute zero by default, and the pricing policy can opt into charging them.",
        defaults_are_free_unless_pricing_policy_says_otherwise,
    );

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

pub const UNCONSUMED_ROOT_FACTOR_IS_INVALID: DescribedBehavior = DescribedBehavior::new(
    "unconsumed root factor is invalid",
    "A factor choice with no priced ancestor is rejected instead of silently disappearing.",
    unconsumed_root_factor_is_invalid,
);

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

fn choice_input_error(error: ChoiceInputError) -> ModifierError {
    error.into()
}

fn cheese_prompt_with_default(prompt_id: ComponentId, default_id: ComponentId) -> Prompt {
    Prompt::new(
        prompt_id,
        "Cheese",
        None,
        vec![Rule::Min(1), Rule::Max(1)],
        Vec::new(),
        vec![
            Choice::new(default_id, "American", vec![Rule::Default(1)], Vec::new()).unwrap(),
            Choice::new(component_id("01SWISS"), "Swiss", Vec::new(), Vec::new()).unwrap(),
        ],
    )
    .unwrap()
}

fn pizza_modifiers() -> Modifiers {
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
                    vec![Rule::Max(1)],
                    Vec::new(),
                )
                .unwrap()
                .with_price(ChoicePrice::flat_amount(usd(200)).unwrap())
                .with_modifiers(Modifiers::new(vec![placement_prompt(component_id(
                    "01PLACE",
                ))])),
            ],
        )
        .unwrap(),
    ])
}

fn pizza_modifiers_with_mushrooms() -> Modifiers {
    let placement = Modifiers::new(vec![placement_prompt(component_id("01PLACE"))]);
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
                .with_modifiers(placement.clone()),
                Choice::new(component_id("01MUSH"), "Mushrooms", Vec::new(), Vec::new())
                    .unwrap()
                    .with_price(ChoicePrice::flat_amount(usd(150)).unwrap())
                    .with_modifiers(placement),
            ],
        )
        .unwrap(),
    ])
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
            .with_modifiers(Modifiers::new(vec![placement_prompt(component_id(
                "01PLACE",
            ))])),
            Choice::new(component_id("01BACON"), "Bacon", Vec::new(), Vec::new())
                .unwrap()
                .with_price(ChoicePrice::flat_amount(usd(300)).unwrap())
                .with_modifiers(Modifiers::new(vec![placement_prompt(component_id(
                    "01PLACE",
                ))])),
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

fn placement_prompt(prompt_id: ComponentId) -> Prompt {
    Prompt::new(
        prompt_id,
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
    .unwrap()
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

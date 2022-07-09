import os
import sys
import datetime
import json
import random
import time
import conditionals as conds
sys.path.append("randomizer")
from randomizer.SettingsList import get_settings_from_section, get_settings_from_tab, get_setting_info
from randomizer.StartingItems import inventory, songs, equipment


def load_weights_file(weights_fname):
    """ Given a weights filename, open it up. If the file does not exist, make it with even weights """
    fpath = os.path.join("weights", weights_fname)
    if os.path.isfile(fpath):
        with open(fpath) as fin:
            datain = json.load(fin)

    weight_options = datain["options"] if "options" in datain else None
    weight_multiselect = datain["multiselect"] if "multiselect" in datain else None
    weight_dict = datain["weights"]

    return weight_options, weight_multiselect, weight_dict


def generate_balanced_weights(fname="default_weights.json"):
    """ Generate a file with even weights for each setting. """
    settings_to_randomize = list(get_settings_from_tab("main_tab"))[1:] + \
                list(get_settings_from_tab("detailed_tab")) + \
                list(get_settings_from_tab("other_tab")) + \
                list(get_settings_from_tab("starting_tab"))

    exclude_from_weights = ["bridge_tokens", "ganon_bosskey_tokens", "triforce_goal_per_world", "disabled_locations",
                            "allowed_tricks", "starting_equipment", "starting_items", "starting_songs"]
    weight_dict = {}
    for name in settings_to_randomize:
        if name not in exclude_from_weights:
            opts = list(get_setting_info(name).choices.keys())
            optsdict = {o: 100./len(opts) for o in opts}
            weight_dict[name] = optsdict

    if fname is not None:
        with open(fname, 'w') as fp:
            json.dump(weight_dict, fp, indent=4)

    return weight_dict


def geometric_weights(N, startat=0, rtype="list"):
    """ Compute weights according to a geometric distribution """
    if rtype == "list":
        return [50.0/2**i for i in range(N)]
    elif rtype == "dict":
        return {str(startat+i): 50.0/2**i for i in range(N)}


def draw_starting_item_pool(random_settings):
    """ Select starting items, songs, and equipment. """
    starting_items = draw_choices_from_pool(inventory)
    starting_songs = draw_choices_from_pool(songs)
    starting_equipment = draw_choices_from_pool(equipment)

    random_settings.setdefault("starting_items", {})
    for pool in (starting_items, starting_songs, starting_equipment):
        for item in pool:
            random_settings["starting_items"].setdefault(item.itemname, 0)
            random_settings["starting_items"][item.itemname] += 1


def draw_choices_from_pool(itempool):
    N = random.choices(range(len(itempool)), weights=geometric_weights(len(itempool)))[0]
    return random.sample(list(itempool.values()), N)


def remove_plando_if_random(random_settings):
    """ For settings that have a _random option, remove the specific plando if _random is true """
    settings_to_check = ["trials", "chicken_count", "big_poe_count"]
    for setting in settings_to_check:
        if random_settings[setting+'_random'] == "true":
            random_settings.pop(setting)


def resolve_multiselect_weights(setting, options):
    """ Given a multiselect weights block, resolve into the plando options. 
    A multiselect block should contain the following elements in addition to individual weights

    global_enable_percentage [0,100] - the chance at rolling any on in the first place
    geometric [true/false] - If true, ignore individual weights and chose a random number
    to enable according to the geometric distribution
    """
    if random.random()*100 > options["global_enable_percentage"]:
        return []
    
    if "geometric" in options.keys() and options["geometric"]:
        nopts = len(get_setting_info(setting).choices)
        N = random.choices(range(nopts+1), weights=geometric_weights(nopts+1))[0]
        return random.sample(list(get_setting_info(setting).choices.keys()), N)

    # Randomly draw which multiselects should be enabled
    if not "opt_percentage" in options.keys():
        return []
    return [msopt for msopt, perc in options["opt_percentage"].items() if random.random()*100 < perc]


def draw_dungeon_shortcuts(random_settings):
    """ Decide how many dungeon shortcuts to enable and randomly select them """
    N = random.choices(range(8), weights=geometric_weights(8))[0]
    dungeon_shortcuts_opts = ['Deku Tree', 'Dodongos Cavern', 'Jabu Jabus Belly', 'Forest Temple', 'Fire Temple', 'Shadow Temple', 'Spirit Temple']
    random_settings["dungeon_shortcuts"] = random.sample(dungeon_shortcuts_opts, N)


def generate_plando(weights, override_weights_fname, no_seed):
    # Load the weight dictionary
    if weights == "RSL":
        weight_options, weight_multiselect, weight_dict = load_weights_file("rsl_season4.json")
    elif weights == "full-random":
        weight_options = None
        weight_dict = generate_balanced_weights(None)
    else:
        weight_options, weight_multiselect, weight_dict = load_weights_file(weights)


    # If an override_weights file name is provided, load it
    start_with = {}
    if override_weights_fname is not None:
        print(f"RSL GENERATOR: LOADING OVERRIDE WEIGHTS from {override_weights_fname}")
        override_options, override_multiselect, override_weights = load_weights_file(override_weights_fname)
        # Check for starting items, songs and equipment
        start_with = override_weights.pop("starting_items", {})

        # Replace the options
        if override_options is not None:
            for key, value in override_options.items():
                # Handling overwrite
                if not (key.startswith("extra_") or key.startswith("remove_")):
                    weight_options[key] = value
                    continue

                # Handling extras
                if key.startswith("extra_"):
                    option = key.split("extra_")[1]
                    if option not in weight_options:
                        weight_options[option] = value
                    else: # Both existing options and extra options
                        if isinstance(weight_options[option], dict):
                            weight_options[option].update(value)
                        else:
                            weight_options[option] += value
                            weight_options[option] = list(set(weight_options[option]))

                # Handling removes
                if key.startswith("remove_"):
                    option = key.split("remove_")[1]
                    if option in weight_options:
                        for item in value:
                            if item in weight_options[option]:
                                weight_options[option].remove(item)

        # Replace the weights
        for key, value in override_weights.items():
            weight_dict[key] = value

        # Replace the multiselects
        if override_multiselect is not None:
            for key, value in override_multiselect.items():
                weight_multiselect[key] = value


    ####################################################################################
    # Make a new function that parses the weights file that does this stuff
    ####################################################################################
    # Generate even weights for tokens and triforce pieces given the max value (Maybe put this into the step that loads the weights)
    for nset in ["bridge_tokens", "ganon_bosskey_tokens", "triforce_goal_per_world"]:
        kw = nset + "_max"
        nmax = weight_options[kw] if kw in weight_options else 100
        weight_dict[nset] = {i+1: 100./nmax for i in range(nmax)}
        if kw in weight_dict:
            weight_dict.pop(kw)
    ####################################################################################

    # Draw the random settings
    random_settings = {"starting_items": start_with}
    for setting, options in weight_dict.items():
        random_settings[setting] = random.choices(list(options.keys()), weights=list(options.values()))[0]

    # Draw the multiselects
    if weight_multiselect is not None:
        for setting, options in weight_multiselect.items():
            random_settings[setting] = resolve_multiselect_weights(setting, options)

    # Add starting items, conditionals, tricks and excluded locations
    if weight_options is not None:
        if "conditionals" in weight_options:
            conds.parse_conditionals(weight_options["conditionals"], weight_dict, random_settings)
        if "tricks" in weight_options:
            random_settings["allowed_tricks"] = weight_options["tricks"]
        if "disabled_locations" in weight_options:
            random_settings["disabled_locations"] = weight_options["disabled_locations"]
        if "starting_items" in weight_options and weight_options["starting_items"] == True:
            draw_starting_item_pool(random_settings)
        
    # Remove plando setting if a _random setting is true
    remove_plando_if_random(random_settings)

    # Format numbers and bools to not be strings
    for setting, value in random_settings.items():
        setting_type = get_setting_info(setting).type
        if setting_type is bool:
            if value == "true":
                value = True
            elif value == "false":
                value = False
            else:
                raise TypeError(f'Value for setting {setting!r} must be "true" or "false"')
        elif setting_type is int:
            value = int(value)
        elif setting_type is not str and setting not in ["allowed_tricks", "disabled_locations", "starting_items", "starting_songs", "starting_equipment", "hint_dist_user", "dungeon_shortcuts"] + list(weight_multiselect.keys()):
            raise NotImplementedError(f'{setting} has an unsupported setting type: {setting_type!r}')
        random_settings[setting] = value

    # Remove conflicting settings
    settings_to_remove = set()
    for setting, choice in random_settings.items():
        info = get_setting_info(setting)
        if info.disable != None:
            for option, disabling in info.disable.items():
                negative = False
                if isinstance(option, str) and option[0] == '!':
                    negative = True
                    option = option[1:]
                if (choice == option) != negative:
                    for other_setting in disabling.get('settings', []):
                        settings_to_remove.add(other_setting)
                    for section in disabling.get('sections', []):
                        for other_setting in get_settings_from_section(section):
                            settings_to_remove.add(other_setting)
                    for tab in disabling.get('tabs', []):
                        for other_setting in get_settings_from_tab(tab):
                            settings_to_remove.add(other_setting)
    for setting_to_remove in settings_to_remove:
        if setting_to_remove in random_settings:
            del random_settings[setting_to_remove]


    # Save the output plando
    output = {"settings": random_settings}

    plando_filename = f'random_settings_{datetime.datetime.utcnow():%Y-%m-%d_%H-%M-%S_%f}.json'
    while os.path.exists(os.path.join('data', plando_filename)):
        time.sleep(0.000001)
        plando_filename = f'random_settings_{datetime.datetime.utcnow():%Y-%m-%d_%H-%M-%S_%f}.json'

    if not os.path.isdir("data"):
        os.mkdir("data")
    with open(os.path.join("data", plando_filename), 'w') as fp:
        json.dump(output, fp, indent=4)
    print(f"Plando File: {plando_filename}")

    return plando_filename

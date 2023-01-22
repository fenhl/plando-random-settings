import random
import json
import os
import sys
from datetime import date, datetime, timedelta
from decimal import Decimal, ROUND_UP


def parse_conditionals(conditional_list, weight_dict, random_settings, extra_starting_items):
    """ Parse the conditionals in the weights file to enable/disable them """
    for cond, details in conditional_list.items():
        if details[0]:
            getattr(sys.modules[__name__], cond)(random_settings, weight_dict=weight_dict, extra_starting_items=extra_starting_items, cparams=details[1:])


def easter_egg_hunt_on_easter(random_settings, **kwargs):
    """ Replace normal Triforce Hunt with Easter Egg Hunt for weights rolled on Easter Sunday or Monday """
    if random_settings['triforce_hunt'] == 'true' and random_settings['triforce_hunt_mode'] == 'normal':
        today = datetime.now().date()
        aa = today.year % 19
        bb = today.year // 100
        cc = today.year % 100
        dd = bb // 4
        ee = bb % 4
        ff = (bb + 8) // 25
        gg = (bb - ff + 1) // 3
        hh = (19 * aa + bb - dd - gg + 15) % 30
        ii = cc // 4
        kk = cc % 4
        ll = (32 + 2 * ee + 2 * ii - hh - kk) % 7
        mm = (aa + 11 * hh + 22 * ll) // 451
        month = (hh + ll - 7 * mm + 114) // 31
        day = (hh + ll - 7 * mm + 114) % 31 + 1
        if date(today.year, month, day) == today or date(today.year, month, day) + timedelta(days=1) == today:
            random_settings['triforce_hunt_mode'] = 'easter_egg_hunt'


def constant_triforce_hunt_extras(random_settings, weight_dict, **kwargs):
    """ Keep constant 25% extra Triforce Pieces for all item pools. """
    random_settings['triforce_count_per_world'] = int(Decimal(random_settings['triforce_goal_per_world'] * 1.25).to_integral_value(rounding=ROUND_UP))


def exclude_minimal_triforce_hunt(random_settings, weight_dict, **kwargs):
    """ If triforce hunt is enabled, reroll the item pool excluding minimal. """
    weights = weight_dict['item_pool_value']
    if 'minimal' in weights.keys() and random_settings['triforce_hunt'] == "true":
        weights.pop('minimal')
    random_settings['item_pool_value'] = random.choices(list(weights.keys()), weights=list(weights.values()))[0]


def exclude_ice_trap_misery(random_settings, weight_dict, **kwargs):
    """ If the damage multiplier is quad or OHKO, exclude ice trap onslaught and mayhem. """
    weights = weight_dict['junk_ice_traps']
    if 'mayhem' in weights.keys() and random_settings['damage_multiplier'] in ['quadruple', 'ohko']:
        weights.pop('mayhem')
    if 'onslaught' in weights.keys() and random_settings['damage_multiplier'] in ['quadruple', 'ohko']:
        weights.pop('onslaught')
    random_settings['junk_ice_traps'] = random.choices(list(weights.keys()), weights=list(weights.values()))[0]


def disable_pot_chest_texture_independence(random_settings, **kwargs):
    """ Set correct_potcrate_appearances to match correct_chest_appearances. """
    if random_settings['correct_chest_appearances'] in ['textures', 'both', 'classic']:
        random_settings['correct_potcrate_appearances'] = 'textures_content'
    else:
        random_settings['correct_potcrate_appearances'] = 'off'


def disable_hideoutkeys_independence(random_settings, **kwargs):
    """ Set shuffle_hideoutkeys to match shuffle_smallkeys. """
    if random_settings['shuffle_smallkeys'] in ['remove', 'vanilla', 'dungeon']:
        random_settings['shuffle_hideoutkeys'] = 'vanilla'
    else:
        random_settings['shuffle_hideoutkeys'] = random_settings['shuffle_smallkeys']


def restrict_one_entrance_randomizer(random_settings, **kwargs):
    erlist = ["shuffle_interior_entrances:off", "shuffle_grotto_entrances:false", "shuffle_dungeon_entrances:false", "shuffle_overworld_entrances:false"]

    # Count how many ER are on
    enabled_er = []
    for item in erlist:
        setting, off_option = item.split(":")
        if random_settings[setting] != off_option:
            enabled_er.append(setting)

    # If too many are enabled, chose one to keep on
    if len(enabled_er) < 2:
        return
    keepon = random.choice(enabled_er).split(":")[0]
    
    # Turn the rest off
    for item in erlist:
        setting, off_option = item.split(":")
        if setting != keepon:
            random_settings[setting] = off_option


def random_scrubs_start_wallet(random_settings, weight_dict, extra_starting_items, **kwargs):
    """ If random scrubs is enabled, add a wallet to the extra starting items """
    if random_settings['shuffle_scrubs'] == 'random':
        extra_starting_items['starting_equipment'] += ['wallet']


def dynamic_skulltula_wincon(random_settings, **kwargs):
    """ Rolls skull win condition seperately. Takes extra inputs [weight of skull win con, "bridge%/gbk%/both"] """
    chance_of_skull_wincon = int(kwargs['cparams'][0])
    weights = [int(x) for x in kwargs['cparams'][1].split('/')]

    # Roll for a skull win condition
    skull_wincon = random.choices([True, False], weights=[chance_of_skull_wincon, 100-chance_of_skull_wincon])[0]
    if not skull_wincon:
        return
    
    # Roll for bridge/bosskey/both
    whichtype = random.choices(['bridge', 'gbk', 'both'], weights=weights)[0]
    if whichtype in ['bridge', 'both']:
        random_settings['bridge'] = 'tokens'
    if whichtype in ['gbk', 'both']:
        random_settings['shuffle_ganon_bosskey'] = 'tokens'


def dynamic_heart_wincon(random_settings, **kwargs):
    """ Rolls heart win condition seperately. Takes extra inputs [weight of skheartull win con, "bridge%/gbk%/both"] """
    chance_of_heart_wincon = int(kwargs['cparams'][0])
    weights = [int(x) for x in kwargs['cparams'][1].split('/')]

    # Roll for a heart win condition
    heart_wincon = random.choices([True, False], weights=[chance_of_heart_wincon, 100-chance_of_heart_wincon])[0]
    if not heart_wincon:
        return

    # Roll for bridge/bosskey/both
    whichtype = random.choices(['bridge', 'gbk', 'both'], weights=weights)[0]
    if whichtype in ['bridge', 'both']:
        random_settings['bridge'] = 'hearts'
    if whichtype in ['gbk', 'both']:
        random_settings['shuffle_ganon_bosskey'] = 'hearts'


def shuffle_goal_hints(random_settings, **kwargs):
    """ Swaps Way of the Hero hints with Goal hints. Takes an extra input [how often to swap] """
    chance_of_goals = int(kwargs['cparams'][0])
    current_distro = random_settings['hint_dist']

    # Roll to swap goal hints
    goals = random.choices([True, False], weights=[chance_of_goals, 100-chance_of_goals])[0]
    if not goals or current_distro == 'useless':
        return

    # Load the distro
    with open(os.path.join('randomizer', 'data', 'Hints', current_distro+'.json')) as fin:
        distroin = json.load(fin)

    # Perform the swap
    woth = {**distroin['distribution']['woth']}
    distroin['distribution']['woth'] = distroin['distribution']['goal']
    distroin['distribution']['goal'] = woth
    random_settings['hint_dist_user'] = distroin


def replace_dampe_diary_hint_with_lightarrow(random_settings, **kwargs):
    """ Replace the dampe diary hint with a Light Arrow hint """
    current_distro = random_settings['hint_dist']

    # Load the distro and change the misc hint
    with open(os.path.join('randomizer', 'data', 'Hints', current_distro+'.json')) as fin:
        distroin = json.load(fin)
    distroin['misc_hint_items'] = { 'dampe_diary': "Light Arrows" }
    random_settings['hint_dist_user'] = distroin



def split_collectible_bridge_conditions(random_settings, **kwargs):
    """ Split heart and skulltula token bridge and ganon boss key.
    kwargs: [how often to have a heart or skull bridge, "heart%/skull%", "bridge%/gbk%/both"]
    """
    chance_of_collectible_wincon = int(kwargs['cparams'][0])
    typeweights = [int(x) for x in kwargs['cparams'][1].split('/')]
    weights = [int(x) for x in kwargs['cparams'][2].split('/')]

    # Roll for collectible win condition
    skull_wincon = random.choices([True, False], weights=[chance_of_collectible_wincon, 100-chance_of_collectible_wincon])[0]
    if not skull_wincon:
        return

    # Roll for hearts or skulls
    condition = random.choices(["hearts", "tokens"], weights=typeweights)[0]
    
    # Roll for bridge/bosskey/both
    whichtype = random.choices(['bridge', 'gbk', 'both'], weights=weights)[0]
    if whichtype in ['bridge', 'both']:
        random_settings['bridge'] = condition
    if whichtype in ['gbk', 'both']:
        random_settings['shuffle_ganon_bosskey'] = condition



def adjust_chaos_hint_distro(random_settings, **kwargs):
    """ Duplicates the always hints in the chaos hint distro and removes
    the double chance at each sometimes hint """

    # Load the dist
    if 'hint_dist_user' in random_settings:
        distroin = random_settings['hint_dist_user']
        if not distroin['name'] == "chaos":
            print("Not using the chaos distribution, passing...")
            return
    else:
        current_distro = random_settings['hint_dist']
        if not current_distro == "chaos":
            print("Not using the chaos distribution, passing...")
            return
        with open(os.path.join('randomizer', 'data', 'Hints', current_distro+'.json')) as fin:
            distroin = json.load(fin)

    # Make changes and save
    distroin['distribution']['always']['copies'] = 2
    distroin['distribution']['sometimes']['weight'] = 0
    random_settings['hint_dist_user'] = distroin



def exclude_mapcompass_info_remove(random_settings, weight_dict, **kwargs):
    """ If Maps and Compai give info, do not allow them to be removed """
    weights = weight_dict['shuffle_mapcompass']
    if 'remove' in weights.keys() and random_settings['enhance_map_compass'] == "true":
        weights.pop('remove')
    random_settings['shuffle_mapcompass'] = random.choices(list(weights.keys()), weights=list(weights.values()))[0]



def ohko_starts_with_nayrus(random_settings, weight_dict, extra_starting_items, **kwargs):
    """ If one hit ko is enabled, add Nayru's Love to the starting items """
    if random_settings['damage_multiplier'] == 'ohko':
        extra_starting_items['starting_items'] += ['nayrus_love']
import sys

import update_randomizer as ur
ur.check_version()

import json

sys.path.append("randomizer")
from randomizer import SettingsList

def sorted_weights(weights):
    sorted_keys = [setting_name for setting_name in SettingsList.si_dict if setting_name in weights['weights']]
    return {**weights, 'weights': {setting_name: weights['weights'][setting_name] for setting_name in sorted_keys}}

if __name__ == '__main__':
    with open('weights/fenhl_override.json', encoding='utf-8') as f:
        weights_text = f.read()
    weights = json.loads(weights_text)
    if '--hook' in sys.argv[1:]:
        if json.dumps(sorted_weights(weights), indent=4) + '\n' != weights_text:
            raise ValueError('weights not sorted correctly, run .\\sort-override.py to fix')
    else:
        with open('weights/fenhl_override.json', 'w', encoding='utf-8') as f:
            json.dump(sorted_weights(weights), f, indent=4)
            print(file=f) # trailing newline

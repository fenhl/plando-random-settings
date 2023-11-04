import sys

import update_randomizer as ur
ur.check_version()

import json

sys.path.append("randomizer")
from randomizer import SettingsList

def sorted_weights(weights):
    return {
        **weights,
        'multiselect': {setting_name: weights['multiselect'][setting_name] for setting_name in SettingsList.SettingInfos.setting_infos if setting_name in weights['multiselect']},
        'weights': {setting_name: weights['weights'][setting_name] for setting_name in SettingsList.SettingInfos.setting_infos if setting_name in weights['weights']},
    }

if __name__ == '__main__':
    for override_name in ('fenhl', 'pictionary'):
        with open(f'weights/{override_name}_override.json', encoding='utf-8') as f:
            weights_text = f.read()
        weights = json.loads(weights_text)
        if '--hook' in sys.argv[1:]:
            if json.dumps(sorted_weights(weights), indent=4) + '\n' != weights_text:
                raise ValueError('weights not sorted correctly, run .\\sort-override.py to fix')
        else:
            with open(f'weights/{override_name}_override.json', 'w', encoding='utf-8') as f:
                json.dump(sorted_weights(weights), f, indent=4)
                print(file=f) # trailing newline

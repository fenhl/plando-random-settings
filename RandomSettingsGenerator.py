""" Run this script to roll a random settings seed! """
import sys
import os
import traceback
import argparse
import shutil

import update_randomizer as ur
ur.check_version()

import rsl_tools as tools
import roll_settings as rs

LOG_ERRORS = True


# Handle all uncaught exceptions with logging
def error_handler(errortype, value, trace):
    """ Custom error handler to write errors to file """
    if LOG_ERRORS:
        with open("ERRORLOG.TXT", 'w') as errout:
            traceback.print_exception(errortype, value, trace, file=errout)
    traceback.print_exception(errortype, value, trace, file=sys.stdout)

    if errortype == tools.RandomizerError:
        sys.exit(3)
sys.excepthook = error_handler


def cleanup(file_to_delete):
    """ Delete residual files that are no longer needed """
    if os.path.isfile(file_to_delete):
        os.remove(file_to_delete)


def get_command_line_args():
    """ Parse the command line arguements """
    global LOG_ERRORS

    parser = argparse.ArgumentParser()
    parser.add_argument("--no_seed", help="Suppresses the generation of a patch file.", action="store_true")
    parser.add_argument("--keep_plandos", help="Don't delete plando files after generating patch files.", action="store_true")
    parser.add_argument("--override", help="Use the specified weights file over the default RSL weights.")
    parser.add_argument("--worldcount", help="Generate a seed with more than 1 world.")
    parser.add_argument("--check_new_settings", help="When the version updates, run with this flag to find changes to settings names or new settings.", action="store_true")
    parser.add_argument("--no_log_errors", help="Only show errors in the console, don't log them to a file.", action="store_true")
    parser.add_argument("--max_plando_retries", help="Try at most this many settings plandos. Defaults to 5.")
    parser.add_argument("--max_rando_retries", help="Try at most this many randomizer runs per settings plando. Defaults to 3.")
    parser.add_argument("--stress_test", help="Generate the specified number of seeds.")
    parser.add_argument("--benchmark", help="Compare the specified weights file to spoiler log empirical data.", action="store_true")
    parser.add_argument("--full_random", help="Allow every setting with even weights.", action="store_true")

    args = parser.parse_args()

    # Parse weights override file
    if args.override is not None:
        if not os.path.isfile(os.path.join("weights", args.override)):
            raise FileNotFoundError("RSL GENERATOR ERROR: CANNOT FIND SPECIFIED OVERRIDE FILE IN DIRECTORY: weights")
        override = args.override
    else:
        override = None

    # Parse integer args
    worldcount = 1
    if args.worldcount is not None:
        worldcount = int(args.worldcount)
    max_plando_retries = 5
    if args.max_plando_retries is not None:
        max_plando_retries = int(args.max_plando_retries)
    max_rando_retries = 3
    if args.max_rando_retries is not None:
        max_rando_retries = int(args.max_rando_retries)

    if args.no_log_errors:
        LOG_ERRORS = False

    seed_count = 1
    if args.stress_test is not None:
        seed_count = int(args.stress_test)

    weights = 'RSL'
    if args.full_random:
        weights = 'full-random'

    return weights, args.no_seed, args.keep_plandos, worldcount, False, override, args.check_new_settings, max_plando_retries, max_rando_retries, seed_count, args.benchmark


def main():
    """ Roll a random settings seed """
    weights, no_seed, keep_plandos, worldcount, per_world_settings, override_weights_fname, check_new_settings, max_plando_retries, max_rando_retries, seed_count, benchmark = get_command_line_args()

    # If we only want to check for new/changed settings
    if check_new_settings:
        _, _, rslweights = rs.load_weights_file("rsl_season5.json")
        tools.check_for_setting_changes(rslweights, rs.generate_balanced_weights(None)[1])
        return

    # If we only want to benchmark weights
    if benchmark:
        weight_options, weight_multiselect, weight_dict, start_with = rs.generate_weights_override(weights, override_weights_fname)
        tools.benchmark_weights(weight_options, weight_dict, weight_multiselect)
        return

    for i in range(seed_count):
        if seed_count > 1:
            print("Rolling test seed", i + 1, "...")

        if LOG_ERRORS:
            # Clean up error log from previous run, if any
            cleanup('ERRORLOG.TXT')

        plandos_to_cleanup = []
        for i in range(max_plando_retries):
            plando_filename = rs.generate_plando(weights, override_weights_fname, no_seed, worldcount if per_world_settings else 1)
            if no_seed:
                # tools.init_randomizer_settings(plando_filename=plando_filename, worldcount=worldcount)
                break
            if not keep_plandos:
                plandos_to_cleanup.append(plando_filename)
            completed_process = tools.generate_patch_file(plando_filename=plando_filename, worldcount=worldcount, max_retries=max_rando_retries)
            if completed_process.returncode == 0:
                break
            if not keep_plandos:
                plandos_to_cleanup.remove(plando_filename)
            if os.path.isfile(os.path.join('data', plando_filename)):
                if not os.path.isdir('failed_settings'):
                    os.mkdir('failed_settings')
                if keep_plandos:
                    shutil.copy(os.path.join('data', plando_filename), os.path.join('failed_settings', plando_filename))
                else:
                    os.rename(os.path.join('data', plando_filename), os.path.join('failed_settings', plando_filename))
            if i == max_plando_retries-1 and completed_process.returncode != 0:
                raise tools.RandomizerError(completed_process.stderr)

        if not no_seed:
            print(completed_process.stderr.split("Patching ROM")[-1])

        for plando_filename in plandos_to_cleanup:
            cleanup(os.path.join('data', plando_filename))


if __name__ == "__main__":
    main()

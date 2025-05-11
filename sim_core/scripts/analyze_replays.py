#!/usr/bin/env python3
import os, glob, json, csv, argparse

# This script parses champion replay JSONL files and extracts health trends per tick/gen

def main():
    parser = argparse.ArgumentParser(description='Analyze NEAT replays')
    parser.add_argument('--replay-dir', default='out', help='Base output directory')
    parser.add_argument('--analysis-dir', default='analysis', help='Directory to write CSV')
    args = parser.parse_args()

    os.makedirs(args.analysis_dir, exist_ok=True)
    rows = []
    pattern = os.path.join(args.replay_dir, 'gen_*/champ_replay.jsonl')
    for path in sorted(glob.glob(pattern)):
        gen = int(path.split('gen_')[1].split(os.sep)[0])
        with open(path) as f:
            for line in f:
                frame = json.loads(line)
                tick = frame['tick']
                agents = frame['agents']
                # health indices based on AGENT_STRIDE=6, IDX_HEALTH=3
                subject_health = agents[3]
                opponent_health = agents[6 + 3] if len(agents) >= 9 else None
                rows.append({'gen': gen, 'tick': tick,
                             'subject_health': subject_health,
                             'opponent_health': opponent_health})
    csv_path = os.path.join(args.analysis_dir, 'health_trends.csv')
    with open(csv_path, 'w', newline='') as csvfile:
        writer = csv.DictWriter(csvfile, fieldnames=['gen','tick','subject_health','opponent_health'])
        writer.writeheader()
        writer.writerows(rows)
    print(f"Wrote {csv_path}")

if __name__ == '__main__':
    main()

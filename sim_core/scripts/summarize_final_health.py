#!/usr/bin/env python3
import csv, os, argparse

def main():
    parser = argparse.ArgumentParser(description='Summarize final champion/opponent health per generation')
    parser.add_argument('--csv', default='analysis/health_trends.csv', help='Health trends CSV')
    parser.add_argument('--out', default='analysis/final_health.csv', help='Output summary CSV')
    args = parser.parse_args()
    # Read trends
    summaries = {}
    with open(args.csv) as f:
        reader = csv.DictReader(f)
        for row in reader:
            gen = int(row['gen'])
            tick = int(row['tick'])
            subj = float(row['subject_health'])
            opp = float(row['opponent_health'])
            # keep the highest tick (final) health
            if gen not in summaries or tick > summaries[gen]['tick']:
                summaries[gen] = {'tick': tick, 'subj': subj, 'opp': opp}
    # Write summary
    os.makedirs(os.path.dirname(args.out), exist_ok=True)
    with open(args.out, 'w', newline='') as f:
        writer = csv.writer(f)
        writer.writerow(['gen','final_subject_health','final_opponent_health'])
        for gen in sorted(summaries):
            writer.writerow([gen,
                             summaries[gen]['subj'],
                             summaries[gen]['opp']])
    print(f"Wrote {args.out}")

if __name__ == '__main__':
    main()

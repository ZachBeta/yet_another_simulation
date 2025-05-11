#!/usr/bin/env python3
import os, csv, argparse

def main():
    parser = argparse.ArgumentParser(description='Summarize match details per generation')
    parser.add_argument('--health-csv', default='analysis/health_trends.csv', help='Health trends CSV')
    parser.add_argument('--out', default='analysis/match_details.csv', help='Output summary CSV')
    args = parser.parse_args()

    # Read health trends
    trends = {}
    with open(args.health_csv) as f:
        reader = csv.DictReader(f)
        for row in reader:
            gen = int(row['gen'])
            tick = int(row['tick'])
            subj = float(row['subject_health'])
            opp = float(row['opponent_health'])
            # track final state
            if gen not in trends or tick > trends[gen]['tick']:
                trends[gen] = {'tick': tick, 'subj': subj, 'opp': opp}

    # Write match details
    os.makedirs(os.path.dirname(args.out), exist_ok=True)
    with open(args.out, 'w', newline='') as f:
        fieldnames = ['gen','final_tick','subject_health','opponent_health','winner','champ_label','opp_label']
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        for gen in sorted(trends):
            data = trends[gen]
            winner = 'draw'
            if data['subj'] > data['opp']:
                winner = 'subject'
            elif data['opp'] > data['subj']:
                winner = 'opponent'
            champ_label = 'hof0'
            opp_label = 'hof1'
            writer.writerow({'gen': gen,
                             'final_tick': data['tick'],
                             'subject_health': data['subj'],
                             'opponent_health': data['opp'],
                             'winner': winner,
                             'champ_label': champ_label,
                             'opp_label': opp_label})
    print(f"Wrote {args.out}")

if __name__ == '__main__':
    main()

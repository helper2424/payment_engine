import random
import csv
from decimal import Decimal

def generate_transactions(num_records=1000000):
    # Transaction types with weights
    tx_types = {
        'deposit': 0.45,      # 45% chance
        'withdrawal': 0.40,   # 40% chance
        'dispute': 0.08,      # 8% chance
        'resolve': 0.04,      # 4% chance
        'chargeback': 0.03    # 3% chance
    }
    
    client_ids = range(1, 2**16 - 1)  # 100 different clients
    tx_id = 1
    deposits_by_client = {}  # Track deposit tx_ids for disputes
    
    with open('transactions.csv', 'w', newline='') as file:
        writer = csv.writer(file)
        writer.writerow(['type', 'client', 'tx', 'amount'])
        
        for _ in range(num_records):
            client_id = random.choice(client_ids)
            tx_type = random.choices(list(tx_types.keys()), 
                                   weights=list(tx_types.values()))[0]
            
            if tx_type == 'deposit':
                amount = round(random.uniform(0.01, 10000.00), 4)
                row = [tx_type, client_id, tx_id, amount]
                deposits_by_client.setdefault(client_id, []).append(tx_id)
            
            elif tx_type == 'withdrawal':
                amount = round(random.uniform(0.01, 1000.00), 4)
                row = [tx_type, client_id, tx_id, amount]
            
            else:  # dispute, resolve, or chargeback
                if client_id in deposits_by_client and deposits_by_client[client_id]:
                    dispute_tx = random.choice(deposits_by_client[client_id])
                    row = [tx_type, client_id, dispute_tx, '']
                else:
                    continue
            
            writer.writerow(row)
            tx_id += 1

generate_transactions()
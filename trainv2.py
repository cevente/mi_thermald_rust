import os
import glob
import pandas as pd
import numpy as np
import xgboost as xgb
from sklearn.model_selection import train_test_split
from sklearn.metrics import mean_absolute_error
import matplotlib.pyplot as plt

# 1. LOAD AND MERGE DATA
def load_data(data_folder="data"):
    print("Loading CSV files...")
    all_files = glob.glob(os.path.join(data_folder, "*.csv"))
    
    df_list = []
    for file in all_files:
        df = pd.read_csv(file)
        df_list.append(df)
        
    full_df = pd.concat(df_list, ignore_index=True)
    
    # Sort by timestamp to keep the time-series order intact
    full_df = full_df.sort_values(by="timestamp_sec").reset_index(drop=True)
    
    print(f"Total samples loaded: {len(full_df)}")
    return full_df

# 2. PREPROCESS DATA (NOW WITH PREDICTIVE SHIFTING)
def preprocess_data(df):
    # Convert temperatures from millidegrees to standard Celsius
    temp_columns = ['t_pa', 't_quiet', 't_charge', 't_emmc', 't_battery']
    for col in temp_columns:
        df[col] = df[col] / 1000.0

    # One-Hot Encode the 'scenario' column
    df = pd.get_dummies(df, columns=['scenario'], drop_first=False)
    
    # =========================================================================
    # TIME-SHIFTING LOGIC (60 Seconds Ahead)
    # Our data is sampled every 2 seconds. 30 rows * 2s = 60 seconds.
    # We shift the target column UP by 30 rows so the model learns to associate 
    # current hardware states with FUTURE temperatures.
    # =========================================================================
    df['target_future_temp'] = df['t_battery'].shift(-30)
    
    # Drop rows with missing data (this will naturally drop the very last 30 rows
    # of each file since they don't have a "future" to look at).
    df = df.dropna()
    
    return df

# 3. TRAIN THE MODEL
def main():
    df = load_data()
    df = preprocess_data(df)
    
    # We drop the timestamp, the CURRENT battery temp, and our new target 
    # so the model can't cheat by looking at the answers.
    features_to_drop = ['timestamp_sec', 't_battery', 'target_future_temp']
    
    # X = Features (Current hardware state)
    X = df.drop(columns=features_to_drop)
    
    # y = Target (Future battery temperature)
    y = df['target_future_temp']
    
    # Split data: 80% for training, 20% for testing
    X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.2, random_state=42, shuffle=True)
    
    print("Training Predictive XGBoost Model (Target: 60s in the future)...")
    model = xgb.XGBRegressor(
        n_estimators=200,      
        learning_rate=0.05,    
        max_depth=6,           
        n_jobs=-1,             
        random_state=42
    )
    
    model.fit(X_train, y_train)
    
    # 4. EVALUATE MODEL
    predictions = model.predict(X_test)
    mae = mean_absolute_error(y_test, predictions)
    print(f"\nModel Mean Absolute Error: {mae:.2f}°C")
    print("This means the model can predict what the temperature will be 1 minute from now, accurate within this many degrees.")

    # 5. VISUALIZE FEATURE IMPORTANCE
    importance = model.feature_importances_
    feature_names = X.columns
    
    plt.figure(figsize=(10, 6))
    sorted_idx = np.argsort(importance)
    plt.barh(range(len(sorted_idx)), importance[sorted_idx], align='center')
    plt.yticks(range(len(sorted_idx)), np.array(feature_names)[sorted_idx])
    plt.title('What Drives FUTURE Battery Heat? (60s Lead Time)')
    plt.xlabel('Importance Score')
    plt.tight_layout()
    
    # Saved under a new name so it doesn't overwrite your first chart
    plt.savefig('feature_importance_predictive.png')
    print("Saved feature importance chart to 'feature_importance_predictive.png'")

if __name__ == "__main__":
    main()

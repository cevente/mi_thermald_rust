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
    
    # Sort by timestamp to keep the time-series order
    full_df = full_df.sort_values(by="timestamp_sec").reset_index(drop=True)
    
    print(f"Total samples loaded: {len(full_df)}")
    return full_df

# 2. PREPROCESS DATA
def preprocess_data(df):
    # Convert temperatures from millidegrees to standard Celsius
    temp_columns = ['t_pa', 't_quiet', 't_charge', 't_emmc', 't_battery']
    for col in temp_columns:
        df[col] = df[col] / 1000.0

    # One-Hot Encode the 'scenario' column (turns 'gaming' into 1s and 0s)
    df = pd.get_dummies(df, columns=['scenario'], drop_first=False)
    
    # Drop rows with missing data
    df = df.dropna()
    
    return df

# 3. TRAIN THE MODEL
def main():
    df = load_data()
    df = preprocess_data(df)
    
    # We don't want to use timestamp as a feature for this basic model
    features_to_drop = ['timestamp_sec', 't_battery']
    
    # X = Features (What the model uses to guess)
    X = df.drop(columns=features_to_drop)
    
    # y = Target (What we are trying to predict - Battery Temp in this case)
    y = df['t_battery']
    
    # Split data: 80% for training, 20% for testing
    X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.2, random_state=42, shuffle=True)
    
    print("Training XGBoost Model...")
    model = xgb.XGBRegressor(
        n_estimators=200,      # Number of trees
        learning_rate=0.05,    # Step size
        max_depth=6,           # Tree depth
        n_jobs=-1,             # Use all CPU cores
        random_state=42
    )
    
    model.fit(X_train, y_train)
    
    # 4. EVALUATE MODEL
    predictions = model.predict(X_test)
    mae = mean_absolute_error(y_test, predictions)
    print(f"\nModel Mean Absolute Error: {mae:.2f}°C")
    print("If this number is e.g. 1.2, it means the model guesses the battery temp accurately within 1.2 degrees.")

    # 5. VISUALIZE FEATURE IMPORTANCE
    # This tells you which hardware sensors/metrics actually drive the heat
    importance = model.feature_importances_
    feature_names = X.columns
    
    plt.figure(figsize=(10, 6))
    sorted_idx = np.argsort(importance)
    plt.barh(range(len(sorted_idx)), importance[sorted_idx], align='center')
    plt.yticks(range(len(sorted_idx)), np.array(feature_names)[sorted_idx])
    plt.title('What Drives Battery Temperature? (Feature Importance)')
    plt.xlabel('Importance Score')
    plt.tight_layout()
    plt.savefig('feature_importance.png')
    print("Saved feature importance chart to 'feature_importance.png'")
    plt.show()

if __name__ == "__main__":
    main()

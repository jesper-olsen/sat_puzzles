import geopandas as gpd
import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches

# --- Your solver’s output (colors) ---
solution = {
    "AK": "G", "AL": "Y", "AR": "Y", "AZ": "Y", "CA": "B", "CO": "Y",
    "CT": "G", "DC": "Y", "DE": "Y", "FL": "R", "GA": "G", "HI": "R",
    "IA": "Y", "ID": "Y", "IL": "B", "IN": "R", "KS": "R", "KY": "Y",
    "LA": "B", "MA": "Y", "MD": "B", "ME": "R", "MI": "G", "MN": "B",
    "MO": "G", "MS": "G", "MT": "B", "NC": "B", "ND": "R", "NE": "B",
    "NH": "G", "NJ": "R", "NM": "G", "NV": "R", "NY": "B", "OH": "B",
    "OK": "B", "OR": "G", "PA": "G", "RI": "R", "SC": "R", "SD": "G",
    "TN": "R", "TX": "R", "UT": "G", "VA": "G", "VT": "R", "WA": "R",
    "WI": "R", "WV": "R", "WY": "R"
}

# 2-letter to full name
state_name_map = {
    "AL": "Alabama", "AK": "Alaska", "AZ": "Arizona", "AR": "Arkansas",
    "CA": "California", "CO": "Colorado", "CT": "Connecticut",
    "DE": "Delaware", "DC": "District of Columbia", "FL": "Florida",
    "GA": "Georgia", "HI": "Hawaii", "ID": "Idaho", "IL": "Illinois",
    "IN": "Indiana", "IA": "Iowa", "KS": "Kansas", "KY": "Kentucky",
    "LA": "Louisiana", "ME": "Maine", "MD": "Maryland", "MA": "Massachusetts",
    "MI": "Michigan", "MN": "Minnesota", "MS": "Mississippi", "MO": "Missouri",
    "MT": "Montana", "NE": "Nebraska", "NV": "Nevada", "NH": "New Hampshire",
    "NJ": "New Jersey", "NM": "New Mexico", "NY": "New York",
    "NC": "North Carolina", "ND": "North Dakota", "OH": "Ohio", "OK": "Oklahoma",
    "OR": "Oregon", "PA": "Pennsylvania", "RI": "Rhode Island",
    "SC": "South Carolina", "SD": "South Dakota", "TN": "Tennessee",
    "TX": "Texas", "UT": "Utah", "VT": "Vermont", "VA": "Virginia",
    "WA": "Washington", "WV": "West Virginia", "WI": "Wisconsin",
    "WY": "Wyoming"
}

# color palette
color_mapping = {
    "R": "#FF6B6B",   # Red
    "G": "#4ECDC4",   # Green
    "B": "#45B7D1",   # Blue
    "Y": "#FFD93D"    # Yellow
}

# Build DataFrame
df = pd.DataFrame([
    {"abbr": abbr, "state": state_name_map[abbr], "color_code": c, "color": color_mapping[c]}
    for abbr, c in solution.items() if abbr in state_name_map
])

# Load US states GeoJSON
url = "https://raw.githubusercontent.com/PublicaMundi/MappingAPI/master/data/geojson/us-states.json"
usa = gpd.read_file(url)

# Merge
merged = usa.merge(df, left_on="name", right_on="state", how="left")
merged["plot_color"] = merged["color"].fillna("lightgrey")  # Grey for unassigned

# Plot
fig, ax = plt.subplots(1, 1, figsize=(14, 10))
merged.plot(ax=ax, color=merged["plot_color"], edgecolor="black")

# Labels
for idx, row in merged.iterrows():
    centroid = row["geometry"].centroid
    label = row["abbr"] if pd.notna(row.get("abbr")) else row["name"]
    ax.annotate(label, (centroid.x, centroid.y),
                ha="center", fontsize=7, weight="bold",
                bbox=dict(boxstyle="round,pad=0.2", facecolor="white", alpha=0.6))

# Legend
legend_items = [mpatches.Patch(color=v, label=k) for k, v in color_mapping.items()]
legend_items.append(mpatches.Patch(color="lightgrey", label="Unassigned"))
plt.legend(handles=legend_items, loc="lower left", title="Colors")

ax.set_title("USA Map Coloring", fontsize=16, weight="bold")
ax.axis("off")
plt.show()

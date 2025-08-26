import geopandas as gpd
import matplotlib.pyplot as plt
import pandas as pd
import matplotlib.patches as mpatches

solution = {
    'NSW': 'B',
    'NT': 'B',
    'QLD': 'R',
    'SA': 'G',
    'TAS': 'R',
    'VIC': 'R',
    'WA': 'R'
}

# Map abbreviations to full names
state_name_map = {
    'NSW': 'New South Wales',
    'NT': 'Northern Territory',
    'QLD': 'Queensland',
    'SA': 'South Australia',
    'TAS': 'Tasmania',
    'VIC': 'Victoria',
    'WA': 'Western Australia',
}

color_mapping = {
    'R': '#FF6B6B',  # Red
    'G': '#4ECDC4',  # Green/Teal
    'B': '#45B7D1',  # Blue
}

solution_df = pd.DataFrame([
    {
        "state_abbr": abbr,
        "state_name": state_name_map[abbr],
        "color_code": code,
        "color": color_mapping[code]
    }
    for abbr, code in solution.items()
])

# --- Load data: states of Australia ---
aus_states = gpd.read_file("https://raw.githubusercontent.com/rowanhogan/australian-states/master/states.geojson")

# Merge shapes + colors
aus_merged = aus_states.merge(solution_df, left_on="STATE_NAME", right_on="state_name", how="left")

# --- Plot ---
fig, ax = plt.subplots(1, 1, figsize=(10, 10))

# Replace NaN in color column with grey
aus_merged["plot_color"] = aus_merged["color"].fillna("lightgrey")

aus_merged.plot(ax=ax, color=aus_merged["plot_color"], edgecolor="black", linewidth=0.8)

# Add state labels
for idx, row in aus_merged.iterrows():
    centroid = row["geometry"].centroid
    label = row["state_abbr"] if pd.notna(row.get("state_abbr")) else row["STATE_NAME"]
    ax.annotate(
        label, (centroid.x, centroid.y),
        ha="center", fontsize=10, weight="bold",
        bbox=dict(boxstyle="round,pad=0.2", facecolor="white", alpha=0.6)
    )

# Legend
import matplotlib.patches as mpatches
legend_items = [
    mpatches.Patch(color=v, label=k) for k, v in color_mapping.items()
]
legend_items.append(mpatches.Patch(color="lightgrey", label="Unassigned"))
plt.legend(handles=legend_items, loc="upper right", title="Colors")

ax.set_title("Australia Map Coloring (3 Colors)", fontsize=14, weight="bold")
ax.axis("off")
plt.show()

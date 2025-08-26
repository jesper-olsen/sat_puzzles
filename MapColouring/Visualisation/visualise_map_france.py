import geopandas as gpd
import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches

solution = {
    "AL": "G", "AQ": "B", "AU": "Y", "BO": "G", "BR": "B",
    "CA": "B", "CE": "B", "CO": "R", "FC": "Y", "IF": "Y",
    "LI": "G", "LO": "R", "LR": "B", "MP": "R", "NB": "R",
    "NH": "G", "NO": "G", "PA": "G", "PC": "R", "PI": "R",
    "PL": "G", "RA": "R",
}


region_map = {
    "AL": "Alsace",
    "AQ": "Aquitaine",
    "AU": "Auvergne",
    "BO": "Bourgogne",
    "BR": "Bretagne",
    "CA": "Champagne-Ardenne",
    "CE": "Centre",
    "FC": "Franche-Comté",
    "IF": "Île-de-France",
    "LI": "Limousin",
    "LO": "Lorraine",
    "LR": "Languedoc-Roussillon",
    "MP": "Midi-Pyrénées",
    "NB": "Basse-Normandie",
    "NH": "Nord-Pas-de-Calais",
    "NO": "Haute-Normandie",
    "PA": "Pays de la Loire",
    "PC": "Poitou-Charentes",
    "PI": "Picardie",
    "PL": "Provence-Alpes-Côte d'Azur",
    "RA": "Rhône-Alpes",
    "CO": "Corse"
}

color_mapping = {
    "R": "#FF6B6B",   # Red
    "G": "#4ECDC4",   # Green
    "B": "#45B7D1",   # Blue
    "Y": "#FFD93D"    # Yellow
}

df = pd.DataFrame([
    {"abbr": abbr,
     "region": region_map[abbr],
     "color_code": c,
     "color": color_mapping[c]}
    for abbr, c in solution.items()
])

# --- Load 22-region France GeoJSON ---
url = "https://raw.githubusercontent.com/gregoiredavid/france-geojson/master/regions-avant-redecoupage-2015.geojson"
france = gpd.read_file(url)

# Merge geodata with colors
merged = france.merge(df, left_on="nom", right_on="region", how="left")

# Fill missing with grey (e.g. Corsica if unassigned)
merged["plot_color"] = merged["color"].fillna("lightgrey")

# --- Plot ---
fig, ax = plt.subplots(1, 1, figsize=(8, 10))
merged.plot(ax=ax, color=merged["plot_color"], edgecolor="black")

# Annotate abbreviations
for idx, row in merged.iterrows():
    centroid = row["geometry"].centroid
    label = row["abbr"] if pd.notna(row.get("abbr")) else row["nom"]
    ax.annotate(label, (centroid.x, centroid.y),
                ha="center", fontsize=7, weight="bold",
                bbox=dict(boxstyle="round,pad=0.2", facecolor="white", alpha=0.6))

# Legend
legend_items = [mpatches.Patch(color=v, label=k) for k, v in color_mapping.items()]
legend_items.append(mpatches.Patch(color="lightgrey", label="Unassigned"))
plt.legend(handles=legend_items, loc="lower left", title="Colors")

ax.set_title("France Map Coloring (Old 22 Regions)", fontsize=14, weight="bold")
ax.axis("off")
plt.show()

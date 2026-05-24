#!/usr/bin/env python3
import customtkinter as ctk
from tkinter import filedialog, messagebox, ttk
import subprocess
import json
import os
import sys

# Define curated theme colors
THEME_DARK_BG = "#121212"
THEME_DARK_CARD = "#1a1a1a"
THEME_DARK_CARD_SEL = "#242424"
THEME_DARK_ACCENT = "#3b82f6"       # Vivid blue
THEME_DARK_SUCCESS = "#10b981"      # Emerald green
THEME_DARK_WARNING = "#f59e0b"      # Amber
THEME_DARK_TEXT_PRIMARY = "#ffffff"
THEME_DARK_TEXT_SECONDARY = "#9ca3af"

THEME_LIGHT_BG = "#f3f4f6"
THEME_LIGHT_CARD = "#ffffff"
THEME_LIGHT_CARD_SEL = "#e5e7eb"
THEME_LIGHT_ACCENT = "#1a73e8"
THEME_LIGHT_SUCCESS = "#059669"
THEME_LIGHT_WARNING = "#d97706"
THEME_LIGHT_TEXT_PRIMARY = "#111827"
THEME_LIGHT_TEXT_SECONDARY = "#4b5563"

class TwinCANApp(ctk.CTk):
    def __init__(self):
        super().__init__()

        self.title("TwinCAN - DBC Comparer")
        self.geometry("1080x700")
        self.minsize(900, 600)
        # Start maximized to fit screen size
        if sys.platform.startswith("win"):
            self.state("zoomed")  # Windows
        else:
            self.attributes("-zoomed", True)  # Linux

        # Set title bar icon
        if getattr(sys, 'frozen', False) and hasattr(sys, '_MEIPASS'):
            base_dir = sys._MEIPASS
        elif '__compiled__' in globals() or hasattr(sys, 'frozen'):
            # Nuitka onefile/standalone fallback. Nuitka usually puts main script at root of distribution.
            if os.path.basename(os.path.dirname(os.path.abspath(__file__))) == "ui":
                base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
            else:
                base_dir = os.path.dirname(os.path.abspath(__file__))
        else:
            base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        icon_path = os.path.join(base_dir, "assets", "icon.ico")
        try:
            if os.path.exists(icon_path):
                self.iconbitmap(icon_path)
        except Exception:
            pass

        # Default settings
        self.current_theme = "Dark"
        ctk.set_appearance_mode(self.current_theme)
        ctk.set_default_color_theme("blue")

        self.dbc1_path = ""
        self.dbc2_path = ""
        self.comparison_data = []
        self.filtered_data = []

        # Find initial examples directory
        self.last_dir = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "examples")
        if not os.path.exists(self.last_dir):
            self.last_dir = os.getcwd()

        # Backend Executable path resolution
        bin_name = "backend.exe" if sys.platform == "win32" else "backend"

        if getattr(sys, 'frozen', False) and hasattr(sys, '_MEIPASS'):
            base_dir = sys._MEIPASS
            self.backend_exe = os.path.join(base_dir, "target", "release", bin_name)
        elif '__compiled__' in globals() or hasattr(sys, 'frozen'):
            # Nuitka onefile/standalone fallback
            if os.path.basename(os.path.dirname(os.path.abspath(__file__))) == "ui":
                base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
            else:
                base_dir = os.path.dirname(os.path.abspath(__file__))
            self.backend_exe = os.path.join(base_dir, "target", "release", bin_name)
        else:
            base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
            # Locally try to find executable in multiple common locations
            possible_paths = [
                os.path.join(base_dir, "target", "release", bin_name),
                os.path.join(base_dir, "target", "debug", bin_name),
                os.path.join(base_dir, bin_name),
                f"/usr/local/bin/{bin_name}",
                rf"C:\Program Files\TwinCAN\{bin_name}"
            ]
            self.backend_exe = None
            for full_path in possible_paths:
                if os.path.exists(full_path):
                    self.backend_exe = full_path
                    break
            
            if not self.backend_exe:
                # Default fallback
                self.backend_exe = os.path.join(base_dir, "target", "release", bin_name)

        self.setup_ui()
        self.apply_theme_colors()

    def setup_ui(self):
        # 1. Main Grid Layout
        self.grid_columnconfigure(0, weight=1)
        self.grid_rowconfigure(2, weight=1)  # Table expands to fill space

        # ---- HEADER BAR ----
        self.header_frame = ctk.CTkFrame(self, fg_color="transparent")
        self.header_frame.grid(row=0, column=0, padx=20, pady=(15, 10), sticky="ew")
        self.header_frame.grid_columnconfigure(0, weight=1)

        self.lbl_title = ctk.CTkLabel(self.header_frame, text="TwinCAN", font=("Segoe UI", 26, "bold"))
        self.lbl_title.grid(row=0, column=0, sticky="w")
        self.lbl_subtitle = ctk.CTkLabel(self.header_frame, text="DBC File Side-by-Side Comparer", font=("Segoe UI", 14), text_color=THEME_DARK_TEXT_SECONDARY)
        self.lbl_subtitle.grid(row=1, column=0, sticky="w")

        # Theme Switcher
        self.theme_toggle = ctk.CTkSwitch(self.header_frame, text="Dark Mode", command=self.toggle_theme, font=("Segoe UI", 14))
        self.theme_toggle.select()
        self.theme_toggle.grid(row=0, column=1, rowspan=2, padx=10, sticky="e")

        # ---- FILE SELECTION PANEL (Side-by-Side Cards) ----
        self.cards_frame = ctk.CTkFrame(self, fg_color="transparent")
        self.cards_frame.grid(row=1, column=0, padx=20, pady=10, sticky="ew")
        self.cards_frame.grid_columnconfigure(0, weight=1)
        self.cards_frame.grid_columnconfigure(1, weight=1)

        # Card 1: DBC File 1
        self.card_dbc1 = ctk.CTkFrame(self.cards_frame, corner_radius=12)
        self.card_dbc1.grid(row=0, column=0, padx=(0, 10), pady=0, sticky="nsew")
        self.card_dbc1.grid_columnconfigure(1, weight=1)

        self.lbl_card1_icon = ctk.CTkLabel(self.card_dbc1, text="📁", font=("Segoe UI", 28))
        self.lbl_card1_icon.grid(row=0, column=0, rowspan=3, padx=(15, 10), pady=15, sticky="n")

        self.lbl_card1_title = ctk.CTkLabel(self.card_dbc1, text="DBC File 1 (Base)", font=("Segoe UI", 16, "bold"))
        self.lbl_card1_title.grid(row=0, column=1, padx=5, pady=(15, 2), sticky="w")

        self.lbl_card1_filename = ctk.CTkLabel(self.card_dbc1, text="Select first DBC file...", font=("Segoe UI", 14, "italic"), text_color=THEME_DARK_TEXT_SECONDARY)
        self.lbl_card1_filename.grid(row=1, column=1, padx=5, pady=2, sticky="w")

        self.lbl_card1_size = ctk.CTkLabel(self.card_dbc1, text="", font=("Segoe UI", 12), text_color=THEME_DARK_TEXT_SECONDARY)
        self.lbl_card1_size.grid(row=2, column=1, padx=5, pady=(0, 15), sticky="w")

        self.btn_browse_dbc1 = ctk.CTkButton(self.card_dbc1, text="Browse", width=95, font=("Segoe UI", 14), command=self.browse_dbc1)
        self.btn_browse_dbc1.grid(row=0, column=3, rowspan=3, padx=15, pady=15, sticky="e")

        self.btn_clear_dbc1 = ctk.CTkButton(self.card_dbc1, text="✕", width=25, height=25, fg_color="transparent", hover_color="#ef4444", text_color="#ef4444", command=self.clear_dbc1)
        # Hidden initially
        
        # Card 2: DBC File 2
        self.card_dbc2 = ctk.CTkFrame(self.cards_frame, corner_radius=12)
        self.card_dbc2.grid(row=0, column=1, padx=(10, 0), pady=0, sticky="nsew")
        self.card_dbc2.grid_columnconfigure(1, weight=1)

        self.lbl_card2_icon = ctk.CTkLabel(self.card_dbc2, text="📁", font=("Segoe UI", 28))
        self.lbl_card2_icon.grid(row=0, column=0, rowspan=3, padx=(15, 10), pady=15, sticky="n")

        self.lbl_card2_title = ctk.CTkLabel(self.card_dbc2, text="DBC File 2 (Target)", font=("Segoe UI", 16, "bold"))
        self.lbl_card2_title.grid(row=0, column=1, padx=5, pady=(15, 2), sticky="w")

        self.lbl_card2_filename = ctk.CTkLabel(self.card_dbc2, text="Select second DBC file...", font=("Segoe UI", 14, "italic"), text_color=THEME_DARK_TEXT_SECONDARY)
        self.lbl_card2_filename.grid(row=1, column=1, padx=5, pady=2, sticky="w")

        self.lbl_card2_size = ctk.CTkLabel(self.card_dbc2, text="", font=("Segoe UI", 12), text_color=THEME_DARK_TEXT_SECONDARY)
        self.lbl_card2_size.grid(row=2, column=1, padx=5, pady=(0, 15), sticky="w")

        self.btn_browse_dbc2 = ctk.CTkButton(self.card_dbc2, text="Browse", width=95, font=("Segoe UI", 14), command=self.browse_dbc2)
        self.btn_browse_dbc2.grid(row=0, column=3, rowspan=3, padx=15, pady=15, sticky="e")

        self.btn_clear_dbc2 = ctk.CTkButton(self.card_dbc2, text="✕", width=25, height=25, fg_color="transparent", hover_color="#ef4444", text_color="#ef4444", command=self.clear_dbc2)

        # ---- RESULTS CONTROL PANEL & GRID ----
        self.results_frame = ctk.CTkFrame(self)
        self.results_frame.grid(row=2, column=0, padx=20, pady=(5, 20), sticky="nsew")
        self.results_frame.grid_columnconfigure(0, weight=1)
        self.results_frame.grid_rowconfigure(1, weight=1) # The grid list fills row 1

        # Control Bar (Search & Filter)
        self.ctrl_bar = ctk.CTkFrame(self.results_frame, fg_color="transparent")
        self.ctrl_bar.grid(row=0, column=0, padx=15, pady=10, sticky="ew")
        self.ctrl_bar.grid_columnconfigure(0, weight=1)

        # Search Entry
        self.entry_search = ctk.CTkEntry(self.ctrl_bar, placeholder_text="🔍 Search message or signal name...", font=("Segoe UI", 14))
        self.entry_search.grid(row=0, column=0, padx=(0, 10), pady=5, sticky="ew")
        self.entry_search.bind("<KeyRelease>", self.on_search_change)

        # Type Filter
        self.lbl_filter_type = ctk.CTkLabel(self.ctrl_bar, text="Type:", font=("Segoe UI", 14))
        self.lbl_filter_type.grid(row=0, column=1, padx=(5, 2), pady=5)
        self.menu_filter_type = ctk.CTkOptionMenu(self.ctrl_bar, values=["All", "Message", "Signal"], width=110, font=("Segoe UI", 14), command=self.on_filter_change)
        self.menu_filter_type.grid(row=0, column=2, padx=5, pady=5)

        # Field Filter
        self.lbl_filter_field = ctk.CTkLabel(self.ctrl_bar, text="Field:", font=("Segoe UI", 14))
        self.lbl_filter_field.grid(row=0, column=3, padx=(5, 2), pady=5)
        self.menu_filter_field = ctk.CTkOptionMenu(self.ctrl_bar, values=["All", "Exists", "DLC / Size", "Start Bit", "Length", "Factor/Offset", "Unit", "Properties"], width=150, font=("Segoe UI", 14), command=self.on_filter_change)
        self.menu_filter_field.grid(row=0, column=4, padx=5, pady=5)

        # Reset Filter Button
        self.btn_reset_filters = ctk.CTkButton(self.ctrl_bar, text="Reset", width=70, fg_color="transparent", text_color=THEME_DARK_TEXT_SECONDARY, border_width=1, border_color="#4b5563", font=("Segoe UI", 14), command=self.reset_filters)
        self.btn_reset_filters.grid(row=0, column=5, padx=5, pady=5)

        # Action Buttons (Compare & Export)
        self.btn_compare = ctk.CTkButton(self.ctrl_bar, text="⚡ Compare Files", font=("Segoe UI", 14, "bold"), fg_color=THEME_DARK_ACCENT, hover_color="#2563eb", command=self.compare_files)
        self.btn_compare.grid(row=0, column=6, padx=5, pady=5)

        self.btn_export = ctk.CTkButton(self.ctrl_bar, text="📊 Export XLSX", font=("Segoe UI", 14, "bold"), fg_color=THEME_DARK_SUCCESS, hover_color="#059669", command=self.export_xlsx, state="disabled")
        self.btn_export.grid(row=0, column=7, padx=(5, 0), pady=5)

        # Modern Zebra Grid using ttk.Treeview wrapped inside custom styled frames
        self.grid_container = ctk.CTkFrame(self.results_frame, fg_color="transparent")
        self.grid_container.grid(row=1, column=0, padx=15, pady=(0, 15), sticky="nsew")
        self.grid_container.grid_columnconfigure(0, weight=1)
        self.grid_container.grid_rowconfigure(0, weight=1)

        # Treeview Styles Configuration
        self.tree_style = ttk.Style()
        self.tree_style.theme_use("clam")
        
        self.tree = ttk.Treeview(
            self.grid_container, 
            columns=("type", "message", "signal", "field", "dbc1", "dbc2"), 
            show="headings", 
            selectmode="browse",
            height=24
        )
        self.tree.grid(row=0, column=0, sticky="nsew")

        # Scrollbars
        self.vsb = ctk.CTkScrollbar(self.grid_container, orientation="vertical", command=self.tree.yview)
        self.vsb.grid(row=0, column=1, sticky="ns")
        self.hsb = ctk.CTkScrollbar(self.grid_container, orientation="horizontal", command=self.tree.xview)
        self.hsb.grid(row=1, column=0, sticky="ew")

        self.tree.configure(yscrollcommand=self.vsb.set, xscrollcommand=self.hsb.set)

        # Define columns and sorting commands
        cols = {
            "type": ("Type", 100),
            "message": ("Message", 240),
            "signal": ("Signal", 240),
            "field": ("Property / Field", 160),
            "dbc1": ("DBC 1 (Base)", 200),
            "dbc2": ("DBC 2 (Target)", 200)
        }

        for cid, (head, width) in cols.items():
            self.tree.heading(cid, text=head, anchor="w", command=lambda c=cid: self.sort_tree_column(c, False))
            self.tree.column(cid, width=width, anchor="w")

        self.tree.bind("<Double-1>", self.on_row_double_click)

        # ---- BOTTOM STATUS BAR ----
        self.status_bar = ctk.CTkFrame(self, height=30, fg_color="transparent")
        self.status_bar.grid(row=3, column=0, sticky="ew")
        
        self.lbl_status = ctk.CTkLabel(self.status_bar, text="Ready", font=("Segoe UI", 13), text_color=THEME_DARK_TEXT_SECONDARY)
        self.lbl_status.pack(side="left", padx=20, pady=5)

    def apply_theme_colors(self):
        # Determine background and secondary colors
        is_dark = self.current_theme == "Dark"
        bg_col = THEME_DARK_BG if is_dark else THEME_LIGHT_BG
        card_col = THEME_DARK_CARD if is_dark else THEME_LIGHT_CARD
        card_sel_col = THEME_DARK_CARD_SEL if is_dark else THEME_LIGHT_CARD_SEL
        text_primary = THEME_DARK_TEXT_PRIMARY if is_dark else THEME_LIGHT_TEXT_PRIMARY
        text_secondary = THEME_DARK_TEXT_SECONDARY if is_dark else THEME_LIGHT_TEXT_SECONDARY
        accent = THEME_DARK_ACCENT if is_dark else THEME_LIGHT_ACCENT
        success = THEME_DARK_SUCCESS if is_dark else THEME_LIGHT_SUCCESS
        warning = THEME_DARK_WARNING if is_dark else THEME_LIGHT_WARNING

        # 1. Main Window
        self.configure(fg_color=bg_col)

        # 2. Card Backgrounds
        self.card_dbc1.configure(fg_color=card_col)
        self.card_dbc2.configure(fg_color=card_col)
        self.results_frame.configure(fg_color=card_col)
        # 3. Label Text Colors
        self.lbl_title.configure(text_color=text_primary)
        self.lbl_subtitle.configure(text_color=text_secondary)
        self.lbl_card1_title.configure(text_color=text_primary)
        self.lbl_card2_title.configure(text_color=text_primary)
        self.lbl_filter_type.configure(text_color=text_primary)
        self.lbl_filter_field.configure(text_color=text_primary)
        self.lbl_status.configure(text_color=text_secondary)
        self.theme_toggle.configure(text_color=text_primary)

        # Handle loaded filename text colors on theme switches
        if self.dbc1_path:
            self.lbl_card1_filename.configure(text_color=accent)
        else:
            self.lbl_card1_filename.configure(text_color=text_secondary)

        if self.dbc2_path:
            self.lbl_card2_filename.configure(text_color=accent)
        else:
            self.lbl_card2_filename.configure(text_color=text_secondary)

        # 4. Inputs & Buttons Theme
        self.entry_search.configure(fg_color=bg_col, text_color=text_primary, border_color="#374151" if is_dark else "#d1d5db")
        self.menu_filter_type.configure(fg_color=bg_col, button_color=bg_col, text_color=text_primary, button_hover_color=card_sel_col)
        self.menu_filter_field.configure(fg_color=bg_col, button_color=bg_col, text_color=text_primary, button_hover_color=card_sel_col)
        self.btn_reset_filters.configure(text_color=text_secondary, border_color="#374151" if is_dark else "#d1d5db")
        self.btn_compare.configure(fg_color=accent, hover_color="#2563eb" if is_dark else "#1a73e8")
        self.btn_export.configure(fg_color=success, hover_color="#059669" if is_dark else "#047857")

        # 5. Treeview Styles
        tree_bg = card_col
        tree_fg = text_primary
        field_bg = card_col
        select_bg = accent
        heading_bg = card_sel_col
        heading_fg = text_primary

        self.tree_style.configure("Treeview", 
            background=tree_bg, 
            foreground=tree_fg, 
            fieldbackground=field_bg, 
            rowheight=38, 
            font=("Segoe UI", 12)
        )
        self.tree_style.map("Treeview", background=[("selected", select_bg)])
        self.tree_style.configure("Treeview.Heading", 
            background=heading_bg, 
            foreground=heading_fg, 
            font=("Segoe UI", 12, "bold"),
            relief="flat"
        )
        self.tree_style.map("Treeview.Heading", 
            background=[("active", card_sel_col)],
            foreground=[("active", heading_fg)]
        )

        # Striping configurations
        even_bg = card_col
        odd_bg = "#222222" if is_dark else "#f9fafb"

        self.tree.tag_configure("evenrow", background=even_bg, foreground=tree_fg)
        self.tree.tag_configure("oddrow", background=odd_bg, foreground=tree_fg)

        # Refresh values to apply color changes
        self.refresh_grid_view()

    def toggle_theme(self):
        if self.theme_toggle.get() == 1:
            self.current_theme = "Dark"
        else:
            self.current_theme = "Light"

        ctk.set_appearance_mode(self.current_theme)
        self.apply_theme_colors()

    def browse_dbc1(self):
        file_path = filedialog.askopenfilename(
            initialdir=self.last_dir,
            title="Select First DBC File (Base)",
            filetypes=[("DBC Files", "*.dbc"), ("All Files", "*.*")]
        )
        if file_path:
            self.last_dir = os.path.dirname(file_path)
            self.set_dbc1(file_path)

    def browse_dbc2(self):
        file_path = filedialog.askopenfilename(
            initialdir=self.last_dir,
            title="Select Second DBC File (Target)",
            filetypes=[("DBC Files", "*.dbc"), ("All Files", "*.*")]
        )
        if file_path:
            self.last_dir = os.path.dirname(file_path)
            self.set_dbc2(file_path)

    def set_dbc1(self, path):
        self.dbc1_path = path
        basename = os.path.basename(path)
        size_str = self.get_file_size_str(path)
        
        self.lbl_card1_filename.configure(text=basename, font=("Segoe UI", 14, "bold"), text_color=THEME_DARK_ACCENT if self.current_theme == "Dark" else THEME_LIGHT_ACCENT)
        self.lbl_card1_size.configure(text=f"Size: {size_str} | Path: {path}")

        # Show clear button to the left of the Browse/Change button
        self.btn_clear_dbc1.grid(row=0, column=2, rowspan=3, padx=(5, 0), pady=15, sticky="e")
        self.btn_browse_dbc1.configure(text="Change")

    def set_dbc2(self, path):
        self.dbc2_path = path
        basename = os.path.basename(path)
        size_str = self.get_file_size_str(path)

        self.lbl_card2_filename.configure(text=basename, font=("Segoe UI", 14, "bold"), text_color=THEME_DARK_ACCENT if self.current_theme == "Dark" else THEME_LIGHT_ACCENT)
        self.lbl_card2_size.configure(text=f"Size: {size_str} | Path: {path}")

        # Show clear button to the left of the Browse/Change button
        self.btn_clear_dbc2.grid(row=0, column=2, rowspan=3, padx=(5, 0), pady=15, sticky="e")
        self.btn_browse_dbc2.configure(text="Change")

    def clear_dbc1(self):
        self.dbc1_path = ""
        self.lbl_card1_filename.configure(text="Select first DBC file...", font=("Segoe UI", 14, "italic"), text_color=THEME_DARK_TEXT_SECONDARY if self.current_theme == "Dark" else THEME_LIGHT_TEXT_SECONDARY)
        self.lbl_card1_size.configure(text="")
        self.btn_clear_dbc1.grid_forget()
        self.btn_browse_dbc1.configure(text="Browse")
        self.reset_comparison_data()

    def clear_dbc2(self):
        self.dbc2_path = ""
        self.lbl_card2_filename.configure(text="Select second DBC file...", font=("Segoe UI", 14, "italic"), text_color=THEME_DARK_TEXT_SECONDARY if self.current_theme == "Dark" else THEME_LIGHT_TEXT_SECONDARY)
        self.lbl_card2_size.configure(text="")
        self.btn_clear_dbc2.grid_forget()
        self.btn_browse_dbc2.configure(text="Browse")
        self.reset_comparison_data()

    def get_file_size_str(self, path):
        try:
            sz = os.path.getsize(path)
            if sz < 1024:
                return f"{sz} B"
            elif sz < 1024 * 1024:
                return f"{sz/1024:.1f} KB"
            else:
                return f"{sz/(1024*1024):.1f} MB"
        except Exception:
            return "Unknown"

    def reset_comparison_data(self):
        self.comparison_data = []
        self.filtered_data = []
        self.tree.delete(*self.tree.get_children())
        self.btn_export.configure(state="disabled")
        
        # Reset column widths to default initial values
        initial_widths = {
            "type": 100,
            "message": 240,
            "signal": 240,
            "field": 160,
            "dbc1": 200,
            "dbc2": 200
        }
        for col, w in initial_widths.items():
            self.tree.column(col, width=w)
        
        self.lbl_status.configure(text="Ready", text_color=THEME_DARK_TEXT_SECONDARY if self.current_theme == "Dark" else THEME_LIGHT_TEXT_SECONDARY)

    def compare_files(self):
        if not self.dbc1_path or not self.dbc2_path:
            messagebox.showwarning("Incomplete Selection", "Please select both DBC files to begin comparison.")
            return

        if not self.backend_exe or not os.path.exists(self.backend_exe):
            messagebox.showerror("Backend Missing", f"Backend comparison executable not found.\nExpected location:\n{self.backend_exe}\n\nPlease verify that the Rust project has been built.")
            return

        self.lbl_status.configure(text="Comparing files... Please wait...", text_color="#3b82f6")
        self.update()

        try:
            result = subprocess.run([self.backend_exe, self.dbc1_path, self.dbc2_path], capture_output=True, text=True)
            if result.returncode != 0:
                self.lbl_status.configure(text="Error in Comparison", text_color="#ef4444")
                messagebox.showerror("Comparison Failed", f"Backend encountered an error:\n{result.stderr}")
                return

            try:
                data = json.loads(result.stdout)
                self.comparison_data = data
                self.btn_export.configure(state="normal")
                self.lbl_status.configure(text=f"Comparison successful. {len(data)} differences identified.", text_color="#10b981")
                
                # Perform initial rendering
                self.apply_search_and_filters()
            except json.JSONDecodeError:
                self.lbl_status.configure(text="Parsing Error", text_color="#ef4444")
                messagebox.showerror("Error Parsing Data", f"Failed to parse comparison backend response:\n{result.stdout}")

        except Exception as e:
            self.lbl_status.configure(text="Execution Failure", text_color="#ef4444")
            messagebox.showerror("Execution Error", f"An exception occurred while executing the backend:\n{str(e)}")

    def apply_search_and_filters(self):
        if not self.comparison_data:
            return

        search_query = self.entry_search.get().strip().lower()
        filter_type = self.menu_filter_type.get()
        filter_field = self.menu_filter_field.get()

        self.filtered_data = []
        for item in self.comparison_data:
            # 1. Type filter
            itype = item.get("result_type", "")
            if filter_type != "All" and itype.lower() != filter_type.lower():
                continue

            # 2. Field filter
            if filter_field != "All":
                field = item.get("field", "").lower()
                if filter_field == "Exists" and "exists" not in field:
                    continue
                elif filter_field == "DLC / Size" and "dlc" not in field and "size" not in field:
                    continue
                elif filter_field == "Start Bit" and "start bit" not in field:
                    continue
                elif filter_field == "Length" and "length" not in field:
                    continue
                elif filter_field == "Factor/Offset" and "factor" not in field and "offset" not in field:
                    continue
                elif filter_field == "Unit" and "unit" not in field:
                    continue
                elif filter_field == "Properties" and ("exists" in field or "dlc" in field or "size" in field or "start bit" in field or "length" in field or "factor" in field or "offset" in field or "unit" in field):
                    continue

            # 3. Search query
            msg = item.get("message", "").lower()
            sig = item.get("signal", "").lower()
            fld = item.get("field", "").lower()
            if search_query and (search_query not in msg and search_query not in sig and search_query not in fld):
                continue

            self.filtered_data.append(item)

        self.refresh_grid_view()

    def refresh_grid_view(self):
        self.tree.delete(*self.tree.get_children())
        
        for idx, item in enumerate(self.filtered_data):
            tag = "evenrow" if idx % 2 == 0 else "oddrow"
            self.tree.insert(
                "", 
                "end", 
                values=(
                    item.get("result_type", ""),
                    item.get("message", ""),
                    item.get("signal", "") or "-",
                    item.get("field", ""),
                    item.get("dbc1", "") or "-",
                    item.get("dbc2", "") or "-"
                ),
                tags=(tag,)
            )
        self.auto_size_columns()

    def auto_size_columns(self):
        import tkinter.font as tkfont
        measure_font = tkfont.Font(family="Segoe UI", size=12)
        header_font = tkfont.Font(family="Segoe UI", size=12, weight="bold")
        
        min_width = 85
        max_width = 450
        padding = 35
        
        columns = self.tree["columns"]
        headers = {
            "type": "Type",
            "message": "Message",
            "signal": "Signal",
            "field": "Property / Field",
            "dbc1": "DBC 1 (Base)",
            "dbc2": "DBC 2 (Target)"
        }
        
        for col in columns:
            header_text = headers.get(col, col)
            max_w = header_font.measure(header_text) + padding
            
            for item in self.tree.get_children():
                val = str(self.tree.set(item, col))
                val_w = measure_font.measure(val) + padding
                if val_w > max_w:
                    max_w = val_w
                    
            final_width = min(max(max_w, min_width), max_width)
            self.tree.column(col, width=final_width)

    def on_search_change(self, event):
        self.apply_search_and_filters()

    def on_filter_change(self, value):
        self.apply_search_and_filters()

    def reset_filters(self):
        self.entry_search.delete(0, "end")
        self.menu_filter_type.set("All")
        self.menu_filter_field.set("All")
        self.apply_search_and_filters()

    def sort_tree_column(self, col, reverse):
        l = [(self.tree.set(k, col), k) for k in self.tree.get_children('')]
        
        # Try numeric sorting if applicable
        try:
            l.sort(key=lambda t: float(t[0]), reverse=reverse)
        except ValueError:
            l.sort(key=lambda t: t[0].lower(), reverse=reverse)

        for index, (val, k) in enumerate(l):
            self.tree.move(k, '', index)

        # Toggle sort order on next click
        self.tree.heading(col, command=lambda: self.sort_tree_column(col, not reverse))

        # Re-apply row striping tags for consistency
        for idx, k in enumerate(self.tree.get_children('')):
            tag = "evenrow" if idx % 2 == 0 else "oddrow"
            self.tree.item(k, tags=(tag,))

    def on_row_double_click(self, event):
        item_id = self.tree.identify_row(event.y)
        if not item_id:
            return

        values = self.tree.item(item_id, "values")
        if not values:
            return

        # Open Custom Detail Window
        detail_window = ctk.CTkToplevel(self)
        detail_window.title("Difference Detailed Inspector")
        detail_window.geometry("600x420")
        detail_window.minsize(550, 380)
        detail_window.transient(self)  # Keep window on top
        detail_window.grab_set()        # Focus locks to this popup

        # Theme matching color choices
        is_dark = self.current_theme == "Dark"
        bg_color = THEME_DARK_BG if is_dark else THEME_LIGHT_BG
        card_color = THEME_DARK_CARD if is_dark else THEME_LIGHT_CARD
        text_primary = THEME_DARK_TEXT_PRIMARY if is_dark else THEME_LIGHT_TEXT_PRIMARY
        text_secondary = THEME_DARK_TEXT_SECONDARY if is_dark else THEME_LIGHT_TEXT_SECONDARY
        accent = THEME_DARK_ACCENT if is_dark else THEME_LIGHT_ACCENT

        detail_window.configure(fg_color=bg_color)

        detail_window.grid_columnconfigure(0, weight=1)
        detail_window.grid_rowconfigure(2, weight=1)

        # Header Title in popup
        header_frame = ctk.CTkFrame(detail_window, fg_color="transparent")
        header_frame.grid(row=0, column=0, padx=20, pady=(15, 10), sticky="ew")

        ctk.CTkLabel(header_frame, text="🔍 Change Inspection", font=("Segoe UI", 18, "bold"), text_color=text_primary).pack(anchor="w")
        ctk.CTkLabel(header_frame, text=f"Type: {values[0]} | Field: {values[3]}", font=("Segoe UI", 13), text_color=text_secondary).pack(anchor="w", pady=(2, 0))

        # Metadata Card
        meta_card = ctk.CTkFrame(detail_window, fg_color=card_color, corner_radius=8)
        meta_card.grid(row=1, column=0, padx=20, pady=5, sticky="ew")
        
        # Message / Signal indicators
        ctk.CTkLabel(meta_card, text="Message Name:", font=("Segoe UI", 13, "bold"), text_color=text_secondary).grid(row=0, column=0, padx=(15, 5), pady=(10, 5), sticky="e")
        ctk.CTkLabel(meta_card, text=values[1], font=("Segoe UI", 13, "bold"), text_color=text_primary).grid(row=0, column=1, padx=5, pady=(10, 5), sticky="w")
        
        if values[2] != "-":
            ctk.CTkLabel(meta_card, text="Signal Name:", font=("Segoe UI", 13, "bold"), text_color=text_secondary).grid(row=1, column=0, padx=(15, 5), pady=(2, 10), sticky="e")
            ctk.CTkLabel(meta_card, text=values[2], font=("Segoe UI", 13, "bold"), text_color=accent).grid(row=1, column=1, padx=5, pady=(2, 10), sticky="w")

        # Visual Side-by-side comparative frames
        comp_frame = ctk.CTkFrame(detail_window, fg_color="transparent")
        comp_frame.grid(row=2, column=0, padx=20, pady=10, sticky="nsew")
        comp_frame.grid_columnconfigure(0, weight=1)
        comp_frame.grid_columnconfigure(1, weight=1)
        comp_frame.grid_rowconfigure(0, weight=1)

        # Left Value Box: DBC 1
        box_dbc1 = ctk.CTkFrame(comp_frame, fg_color=card_color, corner_radius=10)
        box_dbc1.grid(row=0, column=0, padx=(0, 10), pady=0, sticky="nsew")
        box_dbc1.grid_columnconfigure(0, weight=1)
        box_dbc1.grid_rowconfigure(1, weight=1)

        lbl_b1_title = ctk.CTkLabel(box_dbc1, text="DBC 1 (Base)", font=("Segoe UI", 14, "bold"), text_color=THEME_DARK_ACCENT if is_dark else THEME_LIGHT_ACCENT)
        lbl_b1_title.grid(row=0, column=0, padx=10, pady=(10, 5), sticky="nw")

        txt_dbc1 = ctk.CTkTextbox(box_dbc1, font=("Courier New", 14), border_width=0, fg_color="transparent")
        txt_dbc1.grid(row=1, column=0, padx=10, pady=(0, 10), sticky="nsew")
        txt_dbc1.insert("end", values[4])
        txt_dbc1.configure(state="disabled")

        # Right Value Box: DBC 2
        box_dbc2 = ctk.CTkFrame(comp_frame, fg_color=card_color, corner_radius=10)
        box_dbc2.grid(row=0, column=1, padx=(10, 0), pady=0, sticky="nsew")
        box_dbc2.grid_columnconfigure(0, weight=1)
        box_dbc2.grid_rowconfigure(1, weight=1)

        lbl_b2_title = ctk.CTkLabel(box_dbc2, text="DBC 2 (Target)", font=("Segoe UI", 14, "bold"), text_color=THEME_DARK_SUCCESS if is_dark else THEME_LIGHT_SUCCESS)
        lbl_b2_title.grid(row=0, column=0, padx=10, pady=(10, 5), sticky="nw")

        txt_dbc2 = ctk.CTkTextbox(box_dbc2, font=("Courier New", 14), border_width=0, fg_color="transparent")
        txt_dbc2.grid(row=1, column=0, padx=10, pady=(0, 10), sticky="nsew")
        txt_dbc2.insert("end", values[5])
        txt_dbc2.configure(state="disabled")

        # Check for visual states (e.g. green/red highlighting for exists)
        if values[3] == "Exists":
            if values[4].lower() == "yes":
                lbl_b1_title.configure(text="DBC 1 (Exists) ✔", text_color="#10b981")
            else:
                lbl_b1_title.configure(text="DBC 1 (Missing) ✕", text_color="#ef4444")

            if values[5].lower() == "yes":
                lbl_b2_title.configure(text="DBC 2 (Exists) ✔", text_color="#10b981")
            else:
                lbl_b2_title.configure(text="DBC 2 (Missing) ✕", text_color="#ef4444")

        # Footer Options
        footer_frame = ctk.CTkFrame(detail_window, fg_color="transparent")
        footer_frame.grid(row=3, column=0, padx=20, pady=15, sticky="ew")

        def copy_details():
            summary_txt = f"TwinCAN Comparison Details\n--------------------------\nType: {values[0]}\nMessage: {values[1]}\nSignal: {values[2]}\nField: {values[3]}\nBase (DBC1): {values[4]}\nTarget (DBC2): {values[5]}"
            self.clipboard_clear()
            self.clipboard_append(summary_txt)
            self.update()
            btn_copy.configure(text="Copied! ✓", fg_color=THEME_DARK_SUCCESS if is_dark else THEME_LIGHT_SUCCESS)
            detail_window.after(1500, lambda: btn_copy.configure(text="Copy to Clipboard", fg_color=bg_color if is_dark else "#e5e7eb"))

        btn_copy = ctk.CTkButton(footer_frame, text="Copy to Clipboard", width=150, fg_color="transparent", text_color=text_primary, border_width=1, border_color="#4b5563", font=("Segoe UI", 14), command=copy_details)
        btn_copy.pack(side="left")

        btn_close = ctk.CTkButton(footer_frame, text="Close", width=90, font=("Segoe UI", 14), command=detail_window.destroy)
        btn_close.pack(side="right")

    def export_xlsx(self):
        if not self.comparison_data:
            return

        file_path = filedialog.asksaveasfilename(
            initialdir=self.last_dir,
            defaultextension=".xlsx", 
            filetypes=[("Excel Files", "*.xlsx"), ("All Files", "*.*")]
        )
        if not file_path:
            return

        self.lbl_status.configure(text="Exporting XLSX sheet...", text_color="#f59e0b")
        self.update()

        try:
            result = subprocess.run([self.backend_exe, self.dbc1_path, self.dbc2_path, "--export-xlsx", file_path], capture_output=True, text=True)
            if result.returncode != 0:
                self.lbl_status.configure(text="XLSX Export Failed", text_color="#ef4444")
                messagebox.showerror("Export Failed", f"Backend failed to export Excel:\n{result.stderr}")
                return
            
            self.lbl_status.configure(text="XLSX Export Successful.", text_color="#10b981")
            messagebox.showinfo("Export Complete", f"Comparative analysis spreadsheet has been exported to:\n{file_path}")

        except Exception as e:
            self.lbl_status.configure(text="Export Error", text_color="#ef4444")
            messagebox.showerror("Export Error", f"Failed to save Excel sheet:\n{str(e)}")

if __name__ == "__main__":
    app = TwinCANApp()
    app.mainloop()

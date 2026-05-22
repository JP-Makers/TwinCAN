#!/usr/bin/env python3
import customtkinter as ctk
from tkinter import filedialog, messagebox
import subprocess
import json
import os

ctk.set_appearance_mode("Dark")
ctk.set_default_color_theme("blue")

class TwinCANApp(ctk.CTk):
    def __init__(self):
        super().__init__()

        self.title("TwinCAN - DBC Comparer")
        self.geometry("900x600")

        self.dbc1_path = ""
        self.dbc2_path = ""
        self.comparison_data = []

        # Backend Executable path
        import sys
        if getattr(sys, 'frozen', False):
            # If running in a PyInstaller bundle, use the MEIPASS directory
            base_dir = sys._MEIPASS
        else:
            # ui.py is inside ui/, so go up one level to project root
            base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

        self.backend_exe = os.path.join(base_dir, "/usr/local/bin/backend")
        if os.name == 'nt':
            self.backend_exe += ".exe"

        self.setup_ui()

    def setup_ui(self):
        # Grid layout
        self.grid_columnconfigure(0, weight=1)
        self.grid_rowconfigure(2, weight=1)

        # File Selection Frame
        self.file_frame = ctk.CTkFrame(self)
        self.file_frame.grid(row=0, column=0, padx=10, pady=10, sticky="ew")
        self.file_frame.grid_columnconfigure(1, weight=1)

        # DBC 1
        self.lbl_dbc1 = ctk.CTkLabel(self.file_frame, text="DBC 1:")
        self.lbl_dbc1.grid(row=0, column=0, padx=10, pady=10, sticky="w")
        self.entry_dbc1 = ctk.CTkEntry(self.file_frame, placeholder_text="Select first DBC file...")
        self.entry_dbc1.grid(row=0, column=1, padx=10, pady=10, sticky="ew")
        self.btn_dbc1 = ctk.CTkButton(self.file_frame, text="Browse", command=self.browse_dbc1)
        self.btn_dbc1.grid(row=0, column=2, padx=10, pady=10)

        # DBC 2
        self.lbl_dbc2 = ctk.CTkLabel(self.file_frame, text="DBC 2:")
        self.lbl_dbc2.grid(row=1, column=0, padx=10, pady=10, sticky="w")
        self.entry_dbc2 = ctk.CTkEntry(self.file_frame, placeholder_text="Select second DBC file...")
        self.entry_dbc2.grid(row=1, column=1, padx=10, pady=10, sticky="ew")
        self.btn_dbc2 = ctk.CTkButton(self.file_frame, text="Browse", command=self.browse_dbc2)
        self.btn_dbc2.grid(row=1, column=2, padx=10, pady=10)

        # Action Frame
        self.action_frame = ctk.CTkFrame(self)
        self.action_frame.grid(row=1, column=0, padx=10, pady=5, sticky="ew")
        
        self.btn_compare = ctk.CTkButton(self.action_frame, text="Compare Files", command=self.compare_files)
        self.btn_compare.pack(side="left", padx=10, pady=10)

        self.btn_export = ctk.CTkButton(self.action_frame, text="Export XLSX", command=self.export_xlsx, state="disabled")
        self.btn_export.pack(side="left", padx=10, pady=10)

        self.lbl_status = ctk.CTkLabel(self.action_frame, text="Ready", text_color="gray")
        self.lbl_status.pack(side="right", padx=10, pady=10)

        # Results Text (since CustomTkinter doesn't have a Treeview yet, we use a textbox with formatted text)
        self.results_box = ctk.CTkTextbox(self, font=("Courier", 12))
        self.results_box.grid(row=2, column=0, padx=10, pady=10, sticky="nsew")
        self.results_box.configure(state="disabled")

    def browse_dbc1(self):
        file_path = filedialog.askopenfilename(filetypes=[("DBC Files", "*.dbc"), ("All Files", "*.*")])
        if file_path:
            self.dbc1_path = file_path
            self.entry_dbc1.delete(0, "end")
            self.entry_dbc1.insert(0, file_path)

    def browse_dbc2(self):
        file_path = filedialog.askopenfilename(filetypes=[("DBC Files", "*.dbc"), ("All Files", "*.*")])
        if file_path:
            self.dbc2_path = file_path
            self.entry_dbc2.delete(0, "end")
            self.entry_dbc2.insert(0, file_path)

    def display_results(self, data):
        self.results_box.configure(state="normal")
        self.results_box.delete("1.0", "end")
        
        if not data:
            self.results_box.insert("end", "No differences found or invalid data.")
            self.results_box.configure(state="disabled")
            return

        # Header
        header = f"{'TYPE':<10} | {'MESSAGE':<20} | {'SIGNAL':<20} | {'FIELD':<20} | {'DBC1':<25} | {'DBC2':<25}\n"
        separator = "-" * 130 + "\n"
        self.results_box.insert("end", header)
        self.results_box.insert("end", separator)

        for item in data:
            row = f"{item.get('result_type', '')[:10]:<10} | {item.get('message', '')[:20]:<20} | {item.get('signal', '')[:20]:<20} | {item.get('field', '')[:20]:<20} | {item.get('dbc1', '')[:25]:<25} | {item.get('dbc2', '')[:25]:<25}\n"
            self.results_box.insert("end", row)

        self.results_box.configure(state="disabled")

    def compare_files(self):
        dbc1 = self.entry_dbc1.get()
        dbc2 = self.entry_dbc2.get()

        if not dbc1 or not dbc2:
            messagebox.showwarning("Warning", "Please select both DBC files.")
            return

        if not os.path.exists(self.backend_exe):
            messagebox.showerror("Error", f"Backend executable not found at:\n{self.backend_exe}\nPlease build the Rust project first.")
            return

        self.lbl_status.configure(text="Comparing files...", text_color="yellow")
        self.update()

        try:
            result = subprocess.run([self.backend_exe, dbc1, dbc2], capture_output=True, text=True)
            if result.returncode != 0:
                self.lbl_status.configure(text="Error during comparison", text_color="red")
                messagebox.showerror("Error", f"Backend failed:\n{result.stderr}")
                return
            
            # The backend outputs JSON to stdout
            try:
                data = json.loads(result.stdout)
                self.comparison_data = data
                self.display_results(data)
                self.btn_export.configure(state="normal")
                self.lbl_status.configure(text=f"Comparison complete. {len(data)} differences.", text_color="green")
            except json.JSONDecodeError:
                self.lbl_status.configure(text="Error parsing backend output", text_color="red")
                messagebox.showerror("Error", f"Failed to parse backend output:\n{result.stdout}")

        except Exception as e:
            self.lbl_status.configure(text="Execution error", text_color="red")
            messagebox.showerror("Error", str(e))

    def export_xlsx(self):
        if not self.comparison_data:
            return

        dbc1 = self.entry_dbc1.get()
        dbc2 = self.entry_dbc2.get()

        file_path = filedialog.asksaveasfilename(defaultextension=".xlsx", filetypes=[("Excel Files", "*.xlsx")])
        if not file_path:
            return

        self.lbl_status.configure(text="Exporting XLSX...", text_color="yellow")
        self.update()

        try:
            result = subprocess.run([self.backend_exe, dbc1, dbc2, "--export-xlsx", file_path], capture_output=True, text=True)
            if result.returncode != 0:
                self.lbl_status.configure(text="Error exporting XLSX", text_color="red")
                messagebox.showerror("Error", f"Backend failed:\n{result.stderr}")
                return
            
            self.lbl_status.configure(text="XLSX exported successfully.", text_color="green")
            messagebox.showinfo("Success", f"XLSX exported to:\n{file_path}")

        except Exception as e:
            self.lbl_status.configure(text="Export error", text_color="red")
            messagebox.showerror("Error", str(e))

if __name__ == "__main__":
    app = TwinCANApp()
    app.mainloop()

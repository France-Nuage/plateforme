interface Alert {
  id?: number;
  type?: "success" | "danger" | "warning";
  title: string;
  description?: string;
}

export const useAlerts = defineStore("alerts", {
  state: (): { alerts: Array<Alert> } => ({
    alerts: [],
  }),
  getters: {
    list: (state) => state.alerts,
  },
  actions: {
    add(alert: Alert) {
      const id = new Date().getTime();
      this.alerts.push({ ...alert, id });
      this.remove(id);
    },
    remove(id: number) {
      setTimeout(() => {
        this.alerts = this.alerts.filter((_) => _.id !== id);
      }, 5000);
    },
  },
});

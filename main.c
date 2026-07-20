/* aqm-ui : ecran de statut, lance automatiquement en plein ecran par
 * Cage (compositeur Wayland kiosque) au demarrage. Appelle le VRAI
 * aqmctl pour recuperer l'etat du systeme (pas de donnees fictives). */

#include <gtk/gtk.h>
#include <stdio.h>

static char *run_aqmctl(const char *subcmd) {
    char cmd[256];
    snprintf(cmd, sizeof(cmd), "aqmctl %s 2>/dev/null", subcmd);
    FILE *fp = popen(cmd, "r");
    static char buf[4096];
    buf[0] = '\0';
    if (fp) {
        size_t n = fread(buf, 1, sizeof(buf) - 1, fp);
        buf[n] = '\0';
        pclose(fp);
    }
    if (buf[0] == '\0') {
        snprintf(buf, sizeof(buf), "aqm-supervisor indisponible");
    }
    return buf;
}

static gboolean refresh_label(GtkWidget *label) {
    gtk_label_set_text(GTK_LABEL(label), run_aqmctl("status"));
    return G_SOURCE_CONTINUE;
}

static void activate(GtkApplication *app, gpointer user_data) {
    GtkWidget *window = gtk_application_window_new(app);
    gtk_window_set_title(GTK_WINDOW(window), "AsterQuanta OS - Control Center");
    gtk_window_fullscreen(GTK_WINDOW(window));

    GtkWidget *box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 12);
    gtk_widget_set_margin_top(box, 24);
    gtk_widget_set_margin_start(box, 24);

    GtkWidget *title = gtk_label_new("AsterQuanta OS");
    gtk_widget_add_css_class(title, "title-1");

    GtkWidget *status = gtk_label_new(run_aqmctl("status"));
    gtk_label_set_wrap(GTK_LABEL(status), TRUE);

    gtk_box_append(GTK_BOX(box), title);
    gtk_box_append(GTK_BOX(box), status);
    gtk_window_set_child(GTK_WINDOW(window), box);

    g_timeout_add_seconds(5, (GSourceFunc)refresh_label, status);

    gtk_window_present(GTK_WINDOW(window));
}

int main(int argc, char **argv) {
    GtkApplication *app = gtk_application_new(
        "os.asterquanta.ui", G_APPLICATION_DEFAULT_FLAGS);
    g_signal_connect(app, "activate", G_CALLBACK(activate), NULL);
    int status = g_application_run(G_APPLICATION(app), argc, argv);
    g_object_unref(app);
    return status;
}

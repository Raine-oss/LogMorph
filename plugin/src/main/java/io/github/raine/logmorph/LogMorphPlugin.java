// Plugin Main

package io.github.raine.logmorph;

import org.bukkit.Bukkit;
import org.bukkit.ChatColor;
import org.bukkit.command.Command;
import org.bukkit.command.CommandExecutor;
import org.bukkit.command.CommandSender;
import org.bukkit.command.ConsoleCommandSender;
import org.bukkit.command.TabCompleter;
import org.bukkit.entity.Player;
import org.bukkit.plugin.java.JavaPlugin;

import java.io.File;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;

// Plugin Implementation

public class LogMorphPlugin extends JavaPlugin implements CommandExecutor, TabCompleter {

    // Lifecycle

    @Override
    public void onEnable() {
        boolean loaded = LogMorphBridge.loadNative(getDataFolder());
        if (!loaded) {
            getLogger().warning("Failed to load LogMorph native library. Native analysis will be unavailable.");
        } else {
            getLogger().info("LogMorph native engine initialized successfully.");
        }

        if (getCommand("logmorph") != null) {
            getCommand("logmorph").setExecutor(this);
            getCommand("logmorph").setTabCompleter(this);
        }
    }

    @Override
    public void onDisable() {
        getLogger().info("LogMorph plugin disabled.");
    }

    // Command Handler

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        if (!sender.hasPermission("logmorph.admin")) {
            sender.sendMessage(ChatColor.RED + "You do not have permission to use LogMorph.");
            return true;
        }

        if (!LogMorphBridge.isLoaded()) {
            sender.sendMessage(ChatColor.RED + "LogMorph native engine is not loaded. Check server logs.");
            return true;
        }

        boolean summaryOnly = false;
        String pluginFilter = null;

        for (int i = 0; i < args.length; i++) {
            String arg = args[i].toLowerCase();
            if (arg.equals("summary") || arg.equals("--summary") || arg.equals("-s")) {
                summaryOnly = true;
            } else if ((arg.equals("plugin") || arg.equals("--plugin") || arg.equals("-p")) && i + 1 < args.length) {
                pluginFilter = args[++i];
            }
        }

        File serverRoot = getServer().getWorldContainer();
        File logFile = new File(serverRoot, "logs/latest.log");

        if (!logFile.exists()) {
            logFile = new File("logs/latest.log");
        }

        if (!logFile.exists()) {
            sender.sendMessage(ChatColor.RED + "Could not locate logs/latest.log file.");
            return true;
        }

        final String finalLogPath = logFile.getAbsolutePath();
        final boolean finalSummaryOnly = summaryOnly;
        final String finalPluginFilter = pluginFilter;
        final boolean isConsole = sender instanceof ConsoleCommandSender;

        sender.sendMessage(ChatColor.GRAY + "Analyzing server logs with LogMorph...");

        // Async Execution

        Bukkit.getScheduler().runTaskAsynchronously(this, () -> {
            try {
                String result = LogMorphBridge.analyzeLog(
                    finalLogPath,
                    finalSummaryOnly,
                    finalPluginFilter,
                    false
                );

                Bukkit.getScheduler().runTask(this, () -> {
                    if (isConsole) {
                        sender.sendMessage(result);
                    } else {
                        String formatted = formatForChat(result);
                        for (String line : formatted.split("\n")) {
                            if (!line.trim().isEmpty()) {
                                sender.sendMessage(line);
                            }
                        }
                    }
                });
            } catch (Throwable t) {
                Bukkit.getScheduler().runTask(this, () -> {
                    sender.sendMessage(ChatColor.RED + "An error occurred while analyzing logs: " + t.getMessage());
                });
            }
        });

        return true;
    }

    // Chat Formatter

    private String formatForChat(String ansi) {
        String stripped = ansi.replaceAll("\u001B\\[[;\\d]*m", "");
        return ChatColor.DARK_AQUA + "[LogMorph]\n" + ChatColor.WHITE + stripped;
    }

    // Tab Completion

    @Override
    public List<String> onTabComplete(CommandSender sender, Command command, String alias, String[] args) {
        if (args.length == 1) {
            List<String> options = Arrays.asList("summary", "plugin");
            List<String> matches = new ArrayList<>();
            for (String opt : options) {
                if (opt.startsWith(args[0].toLowerCase())) {
                    matches.add(opt);
                }
            }
            return matches;
        }
        return Collections.emptyList();
    }
}

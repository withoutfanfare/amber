<script setup lang="ts">
  import PageHeader from "@/components/layout/PageHeader.vue";
  import { SCard, SBadge } from "@stuntrocket/ui";
</script>

<template>
  <div>
    <PageHeader>
      <template #prepend>
        <h1 class="text-lg font-semibold">Guide</h1>
      </template>
    </PageHeader>

    <div class="stagger-fade-in space-y-6">
      <!-- Getting Started -->
      <SCard>
        <h2 class="mb-3 text-base font-semibold text-text-primary">Getting Started</h2>
        <p class="mb-4 text-sm leading-relaxed text-text-secondary">
          Amber gives you git-like version control for your local development databases. The
          workflow is straightforward:
        </p>
        <ol class="list-inside list-decimal space-y-2 text-sm leading-relaxed text-text-secondary">
          <li>
            <strong class="text-text-primary">Create a connection profile</strong> &mdash; save your
            project connection and its database list once.
          </li>
          <li>
            <strong class="text-text-primary">Take a snapshot</strong> &mdash; capture a compressed
            backup of your database at any point.
          </li>
          <li>
            <strong class="text-text-primary">Restore when needed</strong> &mdash; roll back to any
            previous snapshot instantly.
          </li>
        </ol>
      </SCard>

      <!-- Connection Profiles -->
      <SCard>
        <h2 class="mb-3 text-base font-semibold text-text-primary">Connection Profiles</h2>
        <p class="mb-4 text-sm leading-relaxed text-text-secondary">
          A connection profile represents a project and stores everything Amber needs to connect to
          one or more databases on the same server. This works for projects with a landlord database
          and multiple tenant databases as well as single-database projects.
        </p>

        <h3 class="mb-2 text-xs font-semibold tracking-wide text-text-tertiary uppercase">
          Supported Database Types
        </h3>
        <div class="mb-4 flex gap-2">
          <SBadge variant="accent">MySQL</SBadge>
          <SBadge variant="info">PostgreSQL</SBadge>
          <SBadge variant="default">SQLite</SBadge>
        </div>

        <h3 class="mb-2 text-xs font-semibold tracking-wide text-text-tertiary uppercase">
          Profile Fields
        </h3>
        <ul class="space-y-1.5 text-sm leading-relaxed text-text-secondary">
          <li>
            <strong class="text-text-primary">Name</strong> &mdash; a label to identify this
            connection (e.g. "Local Dev", "Staging Mirror").
          </li>
          <li>
            <strong class="text-text-primary">Project</strong> &mdash; groups related profiles
            together in the sidebar and profile list.
          </li>
          <li>
            <strong class="text-text-primary">Database type</strong> &mdash; determines which
            dump/restore tool is used.
          </li>
          <li>
            <strong class="text-text-primary">Host, port, database names</strong> &mdash; standard
            connection details and every database Amber should snapshot. SQLite profiles use file
            paths instead of database names.
          </li>
          <li>
            <strong class="text-text-primary">Username &amp; password</strong> &mdash; credentials
            are stored securely in your system keychain.
          </li>
        </ul>

        <h3 class="mt-4 mb-2 text-xs font-semibold tracking-wide text-text-tertiary uppercase">
          SSH Tunnels
        </h3>
        <p class="text-sm leading-relaxed text-text-secondary">
          For databases on remote servers, enable the SSH tunnel option. Amber will establish a
          secure tunnel through your SSH host before connecting to the database, so you can snapshot
          remote databases as if they were local.
        </p>
      </SCard>

      <!-- Snapshots -->
      <SCard>
        <h2 class="mb-3 text-base font-semibold text-text-primary">Snapshots</h2>
        <p class="mb-4 text-sm leading-relaxed text-text-secondary">
          A snapshot is a compressed, point-in-time backup of your database. Under the hood, Amber
          uses native database tools (<code class="text-xs text-accent">mysqldump</code>,
          <code class="text-xs text-accent">pg_dump</code>, or SQLite's
          <code class="text-xs text-accent">VACUUM INTO</code>) to produce a SQL dump, then
          compresses it with gzip.
        </p>

        <h3 class="mb-2 text-xs font-semibold tracking-wide text-text-tertiary uppercase">
          How It Works
        </h3>
        <ul class="space-y-1.5 text-sm leading-relaxed text-text-secondary">
          <li>
            <strong class="text-text-primary">Create</strong> &mdash; select a profile and click
            "New Snapshot". Amber creates a separate compressed snapshot for every database in the
            project while showing real-time progress.
          </li>
          <li>
            <strong class="text-text-primary">Test</strong> &mdash; each new snapshot is imported
            into a randomly named temporary local database and then deleted. Configure local MySQL
            or PostgreSQL credentials in Settings; these checks are fixed to
            <code class="text-xs text-accent">127.0.0.1</code> and never use project hosts or SSH.
          </li>
          <li>
            <strong class="text-text-primary">Restore</strong> &mdash; pick any snapshot from the
            list and confirm. Amber decompresses the file and pipes it into the appropriate restore
            tool. This <em>replaces</em> the current database contents.
          </li>
          <li>
            <strong class="text-text-primary">Delete</strong> &mdash; remove a snapshot file from
            disk when you no longer need it.
          </li>
        </ul>

        <h3 class="mt-4 mb-2 text-xs font-semibold tracking-wide text-text-tertiary uppercase">
          File Format
        </h3>
        <p class="text-sm leading-relaxed text-text-secondary">
          Snapshot files are stored as
          <code class="text-xs text-accent">.sql.gz</code> (gzip-compressed SQL). They are organised
          by project in the application data directory and can be inspected with any tool that reads
          gzip files.
        </p>
      </SCard>

      <!-- Storage & Pruning -->
      <SCard>
        <h2 class="mb-3 text-base font-semibold text-text-primary">Storage &amp; Pruning</h2>
        <p class="mb-4 text-sm leading-relaxed text-text-secondary">
          Snapshots live in the application data directory, grouped into folders by project name.
          The Storage view shows a breakdown of disk usage per project so you can see where space is
          being used.
        </p>
        <ul class="space-y-1.5 text-sm leading-relaxed text-text-secondary">
          <li>
            <strong class="text-text-primary">Prune</strong> &mdash; bulk-delete snapshots older
            than a given number of days.
          </li>
          <li>
            <strong class="text-text-primary">Delete by project</strong> &mdash; remove all
            snapshots for a specific project at once.
          </li>
          <li>
            <strong class="text-text-primary">Individual delete</strong> &mdash; remove a single
            snapshot from the Snapshots view.
          </li>
        </ul>
      </SCard>

      <!-- Quick Workflow Tips -->
      <SCard>
        <h2 class="mb-3 text-base font-semibold text-text-primary">Quick Workflow Tips</h2>
        <ul class="space-y-1.5 text-sm leading-relaxed text-text-secondary">
          <li>
            <strong class="text-text-primary">Profile selector</strong> &mdash; use the dropdown at
            the bottom of the sidebar to quickly switch your active profile without leaving the
            current view.
          </li>
          <li>
            <strong class="text-text-primary">Dashboard</strong> &mdash; the home view gives you
            one-click access to common actions: create a snapshot, view recent snapshots, and check
            storage.
          </li>
          <li>
            <strong class="text-text-primary">Filter snapshots</strong> &mdash; on the Snapshots
            view, use the profile filter to narrow the list to a single database.
          </li>
        </ul>
      </SCard>
    </div>
  </div>
</template>

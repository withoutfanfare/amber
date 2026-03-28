export type DbType = "mysql" | "postgresql" | "sqlite";
export type Environment = "local" | "staging" | "live";

export interface Profile {
  id: string;
  project: string;
  name: string;
  dbType: DbType;
  host: string | null;
  port: number | null;
  databaseName: string;
  username: string | null;
  sshEnabled: boolean;
  sshHost: string | null;
  sshPort: number;
  sshUser: string | null;
  environment: Environment | null;
  notes: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface ProfileCreatePayload {
  project: string;
  name: string;
  dbType: DbType;
  host?: string;
  port?: number;
  databaseName: string;
  username?: string;
  password?: string;
  sshEnabled?: boolean;
  sshHost?: string;
  sshPort?: number;
  sshUser?: string;
  sshKeyPath?: string;
  sshPassword?: string;
  environment?: Environment;
  notes?: string;
}

export interface ProfileUpdatePayload extends ProfileCreatePayload {
  id: string;
}

export interface ProfileUpdateInput {
  project?: string;
  name?: string;
  host?: string;
  port?: number;
  databaseName?: string;
  username?: string;
  password?: string;
  sshEnabled?: boolean;
  sshHost?: string;
  sshPort?: number;
  sshUser?: string;
  sshKeyPath?: string;
  sshPassword?: string;
  environment?: Environment;
  notes?: string;
}

export interface Snapshot {
  id: string;
  profileId: string;
  name: string;
  note: string | null;
  filePath: string;
  sizeBytes: number;
  dbVersion: string | null;
  dumpToolVersion: string | null;
  checksum: string | null;
  createdAt: string;
  restoredAt: string | null;
  pinned: boolean;
  tags: string[];
}

export interface SnapshotCreatePayload {
  profileId: string;
  name: string;
  note?: string;
  tags?: string[];
}

export interface RestoreRecord {
  id: string;
  snapshotId: string;
  snapshotName: string;
  targetProfileId: string;
  targetDbName: string;
  durationSecs: number;
  restoredAt: string;
}

export interface SnapshotRestoreOptions {
  targetProfileId?: string;
  targetDatabaseName?: string;
}

export interface StorageUsage {
  project: string;
  totalBytes: number;
  snapshotCount: number;
}

export interface ProjectStorage {
  project: string;
  sizeBytes: number;
  snapshotCount: number;
}

export interface StorageInfo {
  totalBytes: number;
  projects: ProjectStorage[];
}

export interface ConnectionTestResult {
  success: boolean;
  message: string;
  dbVersion?: string;
  latencyMs: number;
  errorKind?: string;
  remediation?: string;
}

export interface IntegrityResult {
  snapshotId: string;
  valid: boolean;
  expectedChecksum: string | null;
  actualChecksum: string | null;
  message: string;
}

export interface SizeEstimation {
  estimatedRawBytes: number;
  estimatedCompressedBytes: number;
  compressionRatio: number;
  estimationMethod: string;
}

export interface DiscoveredTool {
  name: string;
  path: string | null;
  version: string | null;
  found: boolean;
  installHint: string;
  minVersion: string;
}

export interface RetentionPolicy {
  profileId: string;
  maxCount: number | null;
  maxAgeDays: number | null;
  maxSizeBytes: number | null;
}

export interface RetentionEnforcementResult {
  deletedCount: number;
  freedBytes: number;
  reasons: string[];
}

export interface SchemaDiff {
  snapshotAName: string;
  snapshotBName: string;
  tablesAdded: string[];
  tablesRemoved: string[];
  tablesModified: TableDiff[];
  summary: string;
}

export interface TableDiff {
  tableName: string;
  columnsAdded: string[];
  columnsRemoved: string[];
  columnsModified: string[];
}

export interface RestorePreview {
  snapshotName: string;
  snapshotTables: string[];
  currentTables: string[];
  tablesToAdd: string[];
  tablesToRemove: string[];
  tablesInCommon: string[];
  warnings: string[];
}

export interface VersionCompatibility {
  compatible: boolean;
  snapshotToolVersion: string | null;
  snapshotDbVersion: string | null;
  currentToolVersion: string | null;
  warnings: string[];
}

export interface ExportResult {
  outputPath: string;
  sizeBytes: number;
}

export interface HealthCheckResult {
  profileId: string;
  status: "connected" | "unreachable" | "unchecked";
  message: string;
  latencyMs: number;
  checkedAt: string;
}

export interface DiskSpaceInfo {
  availableBytes: number;
  estimatedBytes: number;
  sufficient: boolean;
  safetyMargin: number;
  message: string;
}

export interface OperationStatusResult {
  profileId: string;
  busy: boolean;
  operation: string | null;
}

export type SnapshotProgress =
  | { event: "started"; data: { operation: string; profileName: string } }
  | { event: "phase"; data: { phase: string; message: string } }
  | { event: "progress"; data: { percentage: number; bytesProcessed: number } }
  | {
      event: "tableProgress";
      data: {
        currentTable: string;
        tablesCompleted: number;
        totalTables: number;
        bytesProcessed: number;
      };
    }
  | {
      event: "completed";
      data: { message: string; sizeBytes: number | null; durationSecs: number };
    }
  | { event: "failed"; data: { error: string; phase: string } };

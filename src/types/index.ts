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
}

export interface SnapshotCreatePayload {
  profileId: string;
  name: string;
  note?: string;
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

export type SnapshotProgress =
  | { event: "started"; data: { operation: string; profileName: string } }
  | { event: "phase"; data: { phase: string; message: string } }
  | { event: "progress"; data: { percentage: number; bytesProcessed: number } }
  | {
      event: "completed";
      data: { message: string; sizeBytes: number | null; durationSecs: number };
    }
  | { event: "failed"; data: { error: string; phase: string } };

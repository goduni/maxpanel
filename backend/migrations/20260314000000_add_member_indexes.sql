-- Indexes for efficient user-centric membership lookups
CREATE INDEX IF NOT EXISTS idx_org_members_user ON organization_members (user_id);
CREATE INDEX IF NOT EXISTS idx_project_members_user ON project_members (user_id);

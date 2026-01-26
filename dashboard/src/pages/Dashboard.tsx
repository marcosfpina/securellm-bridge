import { motion } from 'framer-motion'
import {
  Activity,
  FolderKanban,
  Brain,
  AlertTriangle,
  TrendingUp,
  Search,
} from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Progress } from '@/components/ui/progress'
import { Button } from '@/components/ui/button'
import { useStatus, useProjects, useAlerts, useDailyBriefing } from '@/hooks/useApi'
import { getHealthColor, formatRelativeTime } from '@/lib/utils'
import { cn } from '@/lib/utils'

export function Dashboard() {
  const { data: status, isLoading: statusLoading } = useStatus()
  const { data: projects, isLoading: projectsLoading } = useProjects({ sort_by: 'health_score', order: 'asc' })
  const { data: alerts } = useAlerts()
  const { data: briefing } = useDailyBriefing()

  const topProjects = projects?.slice(0, 5) || []
  const recentAlerts = alerts?.slice(0, 5) || []

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Dashboard</h1>
        <p className="text-muted-foreground">
          Central intelligence overview for ~/arch ecosystem
        </p>
      </div>

      {/* Stats Cards */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <StatCard
          title="Total Projects"
          value={status?.total_projects || 0}
          icon={FolderKanban}
          description={`${status?.active_projects || 0} active`}
          loading={statusLoading}
        />
        <StatCard
          title="Intelligence Items"
          value={status?.total_intelligence || 0}
          icon={Brain}
          description="Indexed artifacts"
          loading={statusLoading}
        />
        <StatCard
          title="Ecosystem Health"
          value={`${status?.health_score?.toFixed(0) || 0}%`}
          icon={Activity}
          description="Overall score"
          loading={statusLoading}
          valueClassName={getHealthColor(status?.health_score || 0)}
        />
        <StatCard
          title="Active Alerts"
          value={status?.alerts_count || 0}
          icon={AlertTriangle}
          description="Requires attention"
          loading={statusLoading}
          valueClassName={
            (status?.alerts_count || 0) > 0 ? 'text-red-500' : 'text-green-500'
          }
        />
      </div>

      {/* Main Content Grid */}
      <div className="grid gap-6 lg:grid-cols-2">
        {/* Projects Needing Attention */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between">
            <CardTitle className="text-lg">Projects Needing Attention</CardTitle>
            <Button variant="ghost" size="sm" asChild>
              <a href="/projects">View All</a>
            </Button>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              {projectsLoading ? (
                <div className="text-center text-muted-foreground">Loading...</div>
              ) : topProjects.length === 0 ? (
                <div className="text-center text-muted-foreground">
                  All projects are healthy!
                </div>
              ) : (
                topProjects.map((project) => (
                  <motion.div
                    key={project.name}
                    initial={{ opacity: 0, x: -10 }}
                    animate={{ opacity: 1, x: 0 }}
                    className="flex items-center justify-between rounded-lg border p-3"
                  >
                    <div className="flex-1">
                      <div className="flex items-center gap-2">
                        <span className="font-medium">{project.name}</span>
                        <Badge variant={project.status as any}>
                          {project.status}
                        </Badge>
                      </div>
                      <div className="mt-1 flex items-center gap-4 text-xs text-muted-foreground">
                        <span>{project.languages.slice(0, 2).join(', ')}</span>
                        <span>
                          {formatRelativeTime(project.last_commit)}
                        </span>
                      </div>
                    </div>
                    <div className="flex items-center gap-3">
                      <div className="text-right">
                        <div
                          className={cn(
                            'text-lg font-semibold',
                            getHealthColor(project.health_score)
                          )}
                        >
                          {project.health_score.toFixed(0)}%
                        </div>
                      </div>
                      <Progress
                        value={project.health_score}
                        className="w-20"
                        indicatorClassName={
                          project.health_score >= 70
                            ? 'bg-green-500'
                            : project.health_score >= 50
                            ? 'bg-yellow-500'
                            : 'bg-red-500'
                        }
                      />
                    </div>
                  </motion.div>
                ))
              )}
            </div>
          </CardContent>
        </Card>

        {/* Recent Alerts */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between">
            <CardTitle className="text-lg">Recent Alerts</CardTitle>
            <Badge variant="outline">{alerts?.length || 0} total</Badge>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {recentAlerts.length === 0 ? (
                <div className="text-center text-muted-foreground py-4">
                  No alerts - system is healthy
                </div>
              ) : (
                recentAlerts.map((alert, i) => (
                  <motion.div
                    key={i}
                    initial={{ opacity: 0, x: 10 }}
                    animate={{ opacity: 1, x: 0 }}
                    transition={{ delay: i * 0.05 }}
                    className="flex items-start gap-3 rounded-lg border border-red-500/20 bg-red-500/5 p-3"
                  >
                    <AlertTriangle className="h-4 w-4 shrink-0 text-red-500" />
                    <div className="flex-1 text-sm">
                      <p>{alert.message}</p>
                      {alert.project && (
                        <p className="mt-1 text-xs text-muted-foreground">
                          Project: {alert.project}
                        </p>
                      )}
                    </div>
                  </motion.div>
                ))
              )}
            </div>
          </CardContent>
        </Card>

        {/* Daily Briefing Summary */}
        <Card className="lg:col-span-2">
          <CardHeader className="flex flex-row items-center justify-between">
            <CardTitle className="text-lg">Daily Briefing</CardTitle>
            <Button variant="outline" size="sm" asChild>
              <a href="/briefing">Full Briefing</a>
            </Button>
          </CardHeader>
          <CardContent>
            {briefing ? (
              <div className="space-y-4">
                <div className="rounded-lg bg-muted/50 p-4">
                  <p className="text-sm">{briefing.summary}</p>
                </div>

                {briefing.key_developments && briefing.key_developments.length > 0 && (
                  <div>
                    <h4 className="mb-2 text-sm font-medium">Key Developments</h4>
                    <div className="grid gap-2 md:grid-cols-2">
                      {briefing.key_developments.slice(0, 4).map((dev, i) => (
                        <div
                          key={i}
                          className="flex items-center gap-2 rounded border p-2 text-sm"
                        >
                          <Badge variant={dev.threat_level as any} className="shrink-0">
                            {dev.type}
                          </Badge>
                          <span className="truncate">{dev.title}</span>
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            ) : (
              <div className="text-center text-muted-foreground py-8">
                Loading briefing...
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      {/* Quick Actions */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Quick Actions</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap gap-3">
            <Button variant="outline" asChild>
              <a href="/intelligence">
                <Search className="mr-2 h-4 w-4" />
                Search Intelligence
              </a>
            </Button>
            <Button variant="outline" asChild>
              <a href="/briefing">
                <TrendingUp className="mr-2 h-4 w-4" />
                Executive Summary
              </a>
            </Button>
            <Button variant="outline" asChild>
              <a href="/projects">
                <FolderKanban className="mr-2 h-4 w-4" />
                Browse Projects
              </a>
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

// Stat Card Component
interface StatCardProps {
  title: string
  value: string | number
  icon: React.ElementType
  description: string
  loading?: boolean
  valueClassName?: string
}

function StatCard({
  title,
  value,
  icon: Icon,
  description,
  loading,
  valueClassName,
}: StatCardProps) {
  return (
    <Card>
      <CardContent className="p-6">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm font-medium text-muted-foreground">{title}</p>
            <p className={cn('text-3xl font-bold', valueClassName)}>
              {loading ? '...' : value}
            </p>
            <p className="text-xs text-muted-foreground">{description}</p>
          </div>
          <div className="rounded-full bg-muted p-3">
            <Icon className="h-6 w-6 text-muted-foreground" />
          </div>
        </div>
      </CardContent>
    </Card>
  )
}

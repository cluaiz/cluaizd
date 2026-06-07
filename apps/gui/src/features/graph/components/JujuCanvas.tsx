import React, { useRef, useEffect, useMemo, useState } from 'react';
import * as THREE from 'three';
import * as d3 from 'd3-force-3d';
import SpriteText from 'three-spritetext';
import { useDbStore } from '../../../store/useDbStore';
import ForceGraph3D from 'react-force-graph-3d';

const CLUSTER_ANGLES: Record<string, number> = {
  IDENTITY:  0,
  WORKFORCE: (2 * Math.PI) / 5,
  KNOWLEDGE: (4 * Math.PI) / 5,
  MEMORY:    (6 * Math.PI) / 5,
  REFLEX:    (8 * Math.PI) / 5,
};

const buildSectorForce = () => {
  let nodes: any[] = [];
  const force = (alpha: number) => {
    for (const n of nodes) {
      if (!n.cluster || n.isRoot) continue;
      const targetAngle = CLUSTER_ANGLES[n.cluster] ?? 0;
      const strength = 0.15 * alpha;
      n.vx += Math.cos(targetAngle) * strength;
      n.vz += Math.sin(targetAngle) * strength;
    }
  };
  force.initialize = (n: any[]) => { nodes = n; };
  return force;
};

const LABEL_COLOR: Record<string, string> = {
  OrgNeuron: '#FF2222', Organization: '#FF2222',
  BossNeuron: '#FF4444', DeptNeuron: '#FF6633', CultureNeuron: '#FF8844',
  AgentNeuron: '#FF8C00', Agent: '#FF8C00',
  SkillNeuron: '#FFD700', Skill: '#FFD700', ToolNeuron: '#FFC200',
  PageNeuron: '#22C55E', Document: '#22C55E', File: '#22C55E',
  SessionNode: '#A855F7', ChatSession: '#A855F7',
  DecisionGate: '#3B82F6', Decision: '#3B82F6', TriggerNeuron: '#60A5FA',
};

const getColor = (n: any): string => n.color ?? LABEL_COLOR[n.label ?? ''] ?? '#64748b';

const getSize = (n: any): number => {
  if (n.size) return n.size;
  if (n.isRoot) return 15;
  const l = n.label ?? '';
  if (l === 'OrgNeuron' || l === 'Organization') return 14;
  if (l === 'BossNeuron' || l === 'AgentNeuron') return 9;
  return 4.5;
};

const GEO: Record<string, THREE.BufferGeometry> = {
  sphere: new THREE.SphereGeometry(1, 10, 8),
  dode:   new THREE.DodecahedronGeometry(1),
  oct:    new THREE.OctahedronGeometry(1),
};

const geoOf = (label: string) => {
  if (label === 'OrgNeuron' || label === 'BossNeuron') return GEO.dode;
  if (label === 'AgentNeuron' || label === 'DeptNeuron') return GEO.oct;
  return GEO.sphere;
};

export const JujuCanvas: React.FC = () => {
  const fgRef = useRef<any>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [dim, setDim] = useState({ w: 0, h: 0 });

  const nodes = useDbStore((state) => state.nodes);
  const links = useDbStore((state) => state.links);
  const selectedNode = useDbStore((state) => state.selectedNode);
  const setSelectedNode = useDbStore((state) => state.setSelectedNode);
  const hoveredNode = useDbStore((state) => state.hoveredNode);
  const setHoveredNode = useDbStore((state) => state.setHoveredNode);

  const [hlLinks] = useState(new Set<any>());
  const [hlNodes] = useState(new Set<string>());

  const graphData = useMemo(() => ({ nodes, links }), [nodes, links]);
  const sectorForce = useMemo(() => buildSectorForce(), []);

  // Sync size of container
  useEffect(() => {
    if (!containerRef.current) return;
    const ro = new ResizeObserver(([e]) => {
      setDim({ w: e.contentRect.width, h: e.contentRect.height });
    });
    ro.observe(containerRef.current);
    return () => ro.disconnect();
  }, []);

  // Initialize stars backdrop
  useEffect(() => {
    if (!fgRef.current) return;
    const scene = fgRef.current.scene?.() ?? fgRef.current.scene;
    if (!scene || scene.getObjectByName?.('stars')) return;

    const geo = new THREE.BufferGeometry();
    const pos = new Float32Array(3000 * 3);
    for (let i = 0; i < 3000 * 3; i++) {
      pos[i] = (Math.random() - 0.5) * 4000;
    }
    geo.setAttribute('position', new THREE.BufferAttribute(pos, 3));
    const pts = new THREE.Points(geo, new THREE.PointsMaterial({
      size: 0.9,
      color: 0x39FF14, // Cyber neon Green stars
      transparent: true,
      opacity: 0.2,
      blending: THREE.AdditiveBlending,
      depthWrite: false,
    }));
    pts.name = 'stars';
    scene.add(pts);
  }, [nodes]);

  // Physics config
  useEffect(() => {
    if (!fgRef.current || !nodes.length) return;
    const fg = fgRef.current;
    fg.d3Force('charge', d3.forceManyBody().strength(-200).distanceMax(600));
    fg.d3Force('collide', d3.forceCollide((n: any) => getSize(n) * 3));
    fg.d3Force('sector', sectorForce);
    fg.d3Force('center', null);
    fg.d3AlphaDecay?.(0.02);
    fg.d3VelocityDecay?.(0.4);
    fg.d3ReheatSimulation?.();
  }, [nodes, sectorForce]);

  // Google Maps Style zoom implementation
  useEffect(() => {
    const container = containerRef.current;
    if (!container || !fgRef.current) return;

    const handleWheel = (e: WheelEvent) => {
      const fg = fgRef.current;
      const camera = fg.camera();
      const controls = fg.controls();
      if (!camera || !controls) return;

      e.preventDefault();

      const rect = container.getBoundingClientRect();
      const mouse = new THREE.Vector2(
        ((e.clientX - rect.left) / rect.width) * 2 - 1,
        -((e.clientY - rect.top) / rect.height) * 2 + 1
      );

      const raycaster = new THREE.Raycaster();
      raycaster.setFromCamera(mouse, camera);

      const scene = fg.scene();
      const intersects = raycaster.intersectObjects(scene.children, true);
      const zoomFactor = e.deltaY > 0 ? 1.08 : 0.92;
      const targetPoint = new THREE.Vector3();

      if (intersects.length > 0) {
        targetPoint.copy(intersects[0].point);
      } else {
        const dist = camera.position.distanceTo(controls.target);
        raycaster.ray.at(dist, targetPoint);
      }

      const camPos = camera.position;
      const newCamPos = new THREE.Vector3().lerpVectors(camPos, targetPoint, 1 - zoomFactor);
      const newTarget = new THREE.Vector3().lerpVectors(controls.target, targetPoint, 1 - zoomFactor);

      fg.cameraPosition(newCamPos, newTarget, 0);
    };

    container.addEventListener('wheel', handleWheel, { passive: false });
    return () => container.removeEventListener('wheel', handleWheel);
  }, [nodes]);

  const nodeThreeObject = (raw: any) => {
    const isSel = selectedNode?.id === raw.id;
    const isHov = hoveredNode?.id === raw.id;
    const isNb  = hlNodes.has(raw.id);
    const isConnected = isSel || isHov || isNb;

    // Show label only on select, hover or connected neighbor
    if (!isConnected) return undefined as any;

    const colorHex = getColor(raw);
    const r = getSize(raw);
    const geo = geoOf(raw.label ?? '');
    const mat = new THREE.MeshStandardMaterial({
      color: colorHex,
      emissive: colorHex,
      emissiveIntensity: isConnected ? 2.5 : 0.6,
      metalness: 0.5,
      roughness: 0.3,
    });
    const mesh = new THREE.Mesh(geo, mat);
    mesh.scale.setScalar(r);

    const lbl = new SpriteText(raw.name ?? raw.label ?? '');
    lbl.color = isConnected ? '#ffffff' : `${colorHex}99`;
    lbl.textHeight = (isSel || isHov) ? 12 : 8;
    lbl.position.y = r + 13;
    lbl.backgroundColor = 'rgba(2,2,8,0.9)';
    lbl.padding = [4, 8];
    lbl.borderRadius = 6;
    lbl.fontWeight = '700';

    const group = new THREE.Group();
    group.add(mesh, lbl);
    return group;
  };

  return (
    <div ref={containerRef} className="w-full h-full min-h-[350px] relative bg-cyber-bg border border-cyber-border rounded-xl overflow-hidden glow-blue">
      {dim.w > 0 && nodes.length > 0 ? (
        <ForceGraph3D
          ref={fgRef}
          width={dim.w}
          height={dim.h}
          graphData={graphData}
          backgroundColor="#020208"
          showNavInfo={false}
          nodeLabel=""
          dagMode="radialout"
          dagLevelDistance={150}
          nodeColor={(n: any) => {
            const hex = getColor(n);
            const anyActive = hoveredNode != null || selectedNode != null;
            const isConnected = selectedNode?.id === n.id || hoveredNode?.id === n.id || hlNodes.has(n.id);
            if (anyActive && !isConnected) return `${hex}22`;
            return hex;
          }}
          nodeVal={(n: any) => {
            const r = getSize(n);
            return r * r * 0.4;
          }}
          nodeThreeObject={nodeThreeObject}
          nodeThreeObjectExtend={false}
          linkWidth={(lk: any) => hlLinks.has(lk) ? 2.5 : 1.2}
          linkColor={(lk: any) => {
            const srcColor = getColor(typeof lk.source === 'object' ? lk.source : { label: '' });
            if (hlLinks.has(lk)) return srcColor;
            return '#1f1f3a';
          }}
          linkCurvature={0.08}
          linkDirectionalParticles={(lk: any) => hlLinks.has(lk) ? 4 : 0}
          linkDirectionalParticleWidth={1.5}
          linkDirectionalParticleSpeed={0.01}
          linkDirectionalParticleColor={() => '#ffffff'}
          onNodeClick={(n: any) => {
            if (selectedNode?.id === n.id) {
              setSelectedNode(null);
            } else {
              setSelectedNode(n);
            }
          }}
          onNodeHover={(n: any) => setHoveredNode(n)}
          onBackgroundClick={() => {
            setSelectedNode(null);
            setHoveredNode(null);
          }}
          enableNodeDrag={true}
          onNodeDragEnd={(n: any) => {
            n.fx = n.x; n.fy = n.y; n.fz = n.z;
          }}
          enableNavigationControls={true}
          warmupTicks={50}
          cooldownTicks={100}
        />
      ) : (
        <div className="absolute inset-0 flex flex-col items-center justify-center font-mono text-xs text-cyber-text/50 gap-2">
          <span>Juju Space Empty</span>
          <span className="text-[10px] text-cyber-text/30">Use Ingest form to generate database nodes</span>
        </div>
      )}
    </div>
  );
};
